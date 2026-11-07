use std::cell::BorrowError;
use std::collections::HashMap;
use std::io::{self, Write};
use rusqlite::{params, Connection, Result};

struct Tasks {
    id:i64,
    name:String
}
struct SubTasks{
    id:i64,
    parent_task_id:i64,
    name:String,
    is_ios_done:i64,
    is_android_done:i64,
}
impl Tasks {
    fn add_task(db_conn:&Connection,command:&Vec<&str>)->Self{
        let name: String = command[2..].join(" ");
        let id:i64 = db_conn
            .query_row("INSERT INTO Tasks (name) VALUES (?1) RETURNING id",
                       params![name],|row| row.get(0))
            .expect("could not Insert");
        Self{
            id,
            name
        }
    }
    fn print_tasks(tasks:&Vec<Tasks>){
        clearscreen::clear().expect("Could Not Clear Screen");
        for i in 0..tasks.len(){
            let curr_task = &tasks[i];
            println!("{}: {}",i+1,curr_task.name);
        }
    }
    fn get_tasks(conn:&Connection)->Vec<Tasks>{
        let mut stm = conn.prepare("SELECT id,name FROM Tasks")
            .expect("Could Not execute Select Tasks");
        let mut tasks:Vec<Tasks> = Vec::new();

        let task_iter = stm
            .query_map([], |row|{
                Ok(Tasks{
                    id:row.get(0)?,
                    name:row.get(1)?
                })
            }).expect("could not query Tasks");
        for task in task_iter{
            tasks.push(task.expect("could not get task From task iter"))
        };
        tasks
    }
}
impl SubTasks {
    fn add_sub_task(db_conn:&Connection,command:&Vec<&str>,current_task:&Tasks)->Self{
        let name = command[2..].join(" ");
        let id:i64 = db_conn
            .query_row("INSERT INTO Subtasks (name, task, isIosDone, isAndroidDone)  VALUES (?1,?2,0,0) RETURNING id",
                       params![name,current_task.id], |row| row.get(0))
            .expect("could not Insert sub task");
        Self {
            id,
            parent_task_id:current_task.id,
            name,
            is_ios_done:0,
            is_android_done:0
        }
    }
    fn print_sub_tasks(sub_tasks: &Vec<SubTasks>){
        clearscreen::clear().expect("Could Not Clear Screen");
        for i in 0..sub_tasks.len()  {
            let current_sub_task = &sub_tasks[i];
            println!("{}: {} {} {}",i+1,current_sub_task.name,if current_sub_task.is_ios_done==1{"(IOS Done)"}else{""},if current_sub_task.is_android_done==1{"(Android Done)"}else{""})
        }
    }
    fn get_sub_tasks(conn:&Connection,tasks:&Vec<Tasks>,task_to_be_opened:i32)->Vec<SubTasks>{
        let curr_task = tasks.get(task_to_be_opened as usize);
        let mut sub_tasks:Vec<SubTasks> = Vec::new();
        let mut stm = conn.prepare("SELECT Subtasks.id,t.id,Subtasks.name,isIosDone,isAndroidDone
                                                    FROM Subtasks
                                                    INNER JOIN Tasks t ON Subtasks.task = t.id
                                                    WHERE t.id = ?1")
            .expect("Could perepare stmt  Select SubTasks");
        if curr_task.is_some() {
            let curr_task = curr_task.unwrap();
            let sub_task_iter = stm
                .query_map([curr_task.id],|row|{
                    Ok(
                        SubTasks{
                            id:row.get(0)?,
                            parent_task_id:row.get(1)?,
                            name:row.get(2)?,
                            is_ios_done:row.get(3)?,
                            is_android_done:row.get(4)?
                        }
                    )
                }).expect("could not query Sub Tasks");
            for st in sub_task_iter {
                sub_tasks.push(st.expect("could not Get subtTusck From loop"));
            }
            SubTasks::print_sub_tasks(&sub_tasks);
            sub_tasks
        }else {
            println!("trying to open none existing Task");
            sub_tasks
        }
    }
    fn complete_android(&mut self, db_conn:&Connection){
        db_conn
            .execute("UPDATE Subtasks SET isAndroidDone = 1 WHERE id = ?1",params![self.id])
            .expect("could not update subtask");
        self.is_android_done = 1;
    }
    fn complete_ios(&mut self, db_conn:&Connection){
        db_conn
            .execute("UPDATE Subtasks SET isIosDone = 1 WHERE id = ?1",params![self.id])
            .expect("could not update subtask");
        self.is_ios_done = 1;
    }
}
fn main() {
    let mut commands = HashMap::new();
    commands.insert("help","Shows All Of The Existing Commands");
    commands.insert("add -t {argument}","Adds Task With The Name Of Argument");
    commands.insert("open {argument}","Opens task with id that is being passed in argument");
    commands.insert("done -a {argument}","Finishes Android on a sub task with id that is being passed in argument");
    commands.insert("done -i {argument}","Finishes IOS on a sub task with id that is being passed in argument");
    commands.insert("add -s {argument}","Adds SubTask to a currently openedTask With The Name Of Argument");
    commands.insert("exit","exits out from process");
    commands.insert("remove -a {argument}","Removes Android on a sub task with id that is being passed in argument");
    commands.insert("remove -i {argument}","Removes IOS on a sub task with id that is being passed in argument");
    commands.insert("del -t {argument}","deletes task with id and its all of subtasks");
    commands.insert("show","shows all tasks");
    let db_conn = create_database_con().expect("Could Not Open DataBase");
    let mut current_tasks:Vec<Tasks> = Tasks::get_tasks(&db_conn);
    let mut current_sub_tasks:Vec<SubTasks> = Vec::new();
    let mut current_opened_task = -1;
    Tasks::print_tasks(&current_tasks);

    loop {
        print!("Command: ");
        io::stdout().flush().expect("Failed to flush stdout");
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed To Read User Line");

        let commands_as_vec:Vec<&str> = input.trim().split(" ").collect();
        let curr_command = commands_as_vec.get(0);
        match curr_command.copied() {
            Some("help")=>handle_help_command(&commands),
            Some("add")=> {
                handle_add_task(&mut current_tasks,&db_conn,&commands_as_vec,current_opened_task,&mut current_sub_tasks);
            }
            Some("open")=>{
                current_opened_task = commands_as_vec[1].parse::<i32>().unwrap() -1;
                current_sub_tasks = SubTasks::get_sub_tasks(&db_conn,&current_tasks,current_opened_task);
            }
            Some("exit")=>{
                break;
            }
            Some("done")=>handle_done_sub_tasks(&mut current_sub_tasks,&db_conn,&commands_as_vec),
            Some("remove")=>handle_remove_sub_tasks(&mut current_sub_tasks,&db_conn,&commands_as_vec),
            Some("del")=> handle_delete_task(&mut current_tasks,&db_conn,&commands_as_vec,&mut current_sub_tasks),
            Some("show")=>Tasks::print_tasks(&current_tasks),
            _ => println!("Not A Valid Command")
        }
    }
    db_conn.close().expect("Failed to Close Database");
}
fn handle_help_command(commands: &HashMap<&str,&str>){
    clearscreen::clear().expect("Could Not Clear Screen");
    for (command,explanation) in commands {
        println!("{} : {}",command,explanation)
    }
}
fn handle_add_task(tasks:&mut Vec<Tasks>,db_conn:&Connection,command:&Vec<&str>,current_open_task_i:i32,sub_tasks:&mut Vec<SubTasks>){
    match command.get(1).copied() {
        Some("-t")=>{
            let task_name = command.get(2);
            if task_name.is_some() {
                tasks.push(Tasks::add_task(db_conn,command));
                Tasks::print_tasks(&tasks);
            }else {
                println!("Task Name Needed");
            }
        },
        Some("-s")=>{
            let current_task = tasks.get(current_open_task_i as usize).unwrap();
            sub_tasks.push(SubTasks::add_sub_task(db_conn,command,current_task));
            SubTasks::print_sub_tasks(sub_tasks)
        }
        _ => println!("Not A Valid Add Argument")
    }
}
fn create_database_con()-> Result<Connection>{
    let conn = Connection::open("tasks.db")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    Ok(conn)
}
fn handle_done_sub_tasks(sub_tasks: &mut Vec<SubTasks>,db_conn:&Connection,command:&Vec<&str>){
    match command.get(1).copied() {
        Some("-a")=>{
            let current_sub_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_sub_task = sub_tasks.get_mut(current_sub_task_i);
            if current_sub_task.is_some(){
                current_sub_task.unwrap().complete_android(db_conn);
            }
            SubTasks::print_sub_tasks(sub_tasks);
        },
        Some("-i")=>{
            let current_sub_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_sub_task = sub_tasks.get_mut(current_sub_task_i);
            if current_sub_task.is_some(){
                current_sub_task.unwrap().complete_ios(db_conn);
            }
            SubTasks::print_sub_tasks(sub_tasks);
        }
        _=>{
            println!("Task Name Needed");
        }
    }
}
fn handle_remove_sub_tasks(sub_tasks: &mut Vec<SubTasks>,db_conn:&Connection,command:&Vec<&str>){
    match command.get(1).copied() {
        Some("-a")=>{
            let current_sub_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_sub_task = sub_tasks.get(current_sub_task_i);
            if current_sub_task.is_some(){
                db_conn
                    .execute("UPDATE Subtasks SET isAndroidDone = 0 WHERE id = ?1",params![current_sub_task.unwrap().id])
                    .expect("could not update subtask");
            }
            let new_sub_task = sub_tasks.get_mut(current_sub_task_i).unwrap();
            new_sub_task.is_android_done = 0;
            SubTasks::print_sub_tasks(sub_tasks);
        },
        Some("-i")=>{
            let current_sub_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_sub_task = sub_tasks.get(current_sub_task_i);
            if current_sub_task.is_some(){
                db_conn
                    .execute("UPDATE Subtasks SET isIosDone = 0 WHERE id = ?1",params![current_sub_task.unwrap().id])
                    .expect("could not update subtask");
            }
            let new_sub_task = sub_tasks.get_mut(current_sub_task_i).unwrap();
            new_sub_task.is_ios_done = 0;
            SubTasks::print_sub_tasks(sub_tasks);
        }
        _=>{
            println!("Task Id Needed");
        }
    }
}
fn handle_delete_task(tasks:&mut Vec<Tasks>,db_conn:&Connection,command:&Vec<&str>,sub_tasks:&mut Vec<SubTasks>){
    match command.get(1).copied() {
        Some("-t")=>{
            let current_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_task = tasks.get(current_task_i);
            if current_task.is_some(){
                db_conn
                    .execute("DELETE FROM Tasks WHERE id = ?1",params![current_task.unwrap().id])
                    .expect("could not delete task");
            }
            tasks.remove(current_task_i);
            Tasks::print_tasks(&tasks);
        }
        Some("-s")=> {
            let current_sub_task_i = command.get(2).unwrap().parse::<usize>().unwrap()-1;
            let current_sub_task = sub_tasks.get(current_sub_task_i);
            if current_sub_task.is_some(){
                db_conn
                    .execute("DELETE FROM Subtasks WHERE id = ?1",params![current_sub_task.unwrap().id])
                    .expect("could not delete task");
            }
            sub_tasks.remove(current_sub_task_i);
            SubTasks::print_sub_tasks(sub_tasks);
        }
        _=>{
            println!("Task Id Needed");
        }
    }
}