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
fn main() {
    let mut commands = HashMap::new();
    commands.insert("help","Shows All Of The Existing Commands");
    commands.insert("add -t {argument}","Adds Task With The Name Of Argument");
    commands.insert("open {argument}","Opens task with id that is being passed in argument");
    commands.insert("done -a {argument}","Finishes Android on a task with id that is being passed in argument");
    commands.insert("add -s {argument}","Adds SubTask to a currently openedTask With The Name Of Argument");
    let db_conn = create_database_con().expect("Could Not Open DataBase");
    let mut current_tasks:Vec<Tasks> = get_tasks(&db_conn);
    let mut current_sub_tasks:Vec<SubTasks> = Vec::new();
    let mut current_opened_task = -1;
    print_tasks(&current_tasks);

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
                handle_add_task(&mut current_tasks,&db_conn,&commands_as_vec);
            }
            Some("open")=>{
                current_opened_task = commands_as_vec[1].parse().unwrap();
                current_sub_tasks = get_sub_tasks(&db_conn,&current_tasks,current_opened_task);
            }
            _ => println!("Not A Valid Command")
        }
    }
}
fn handle_help_command(commands: &HashMap<&str,&str>){
    clearscreen::clear().expect("Could Not Clear Screen");
    for (command,explanation) in commands {
        println!("{} : {}",command,explanation)
    }
}
fn handle_add_task(tasks:&mut Vec<Tasks>,db_conn:&Connection,command:&Vec<&str>){
    match command.get(1).copied() {
        Some("-t")=>{
            let task_name = command.get(2);
            if task_name.is_some() {
                let task_name = *command.get(2).unwrap();
                db_conn
                    .execute("INSERT INTO Tasks (name) VALUES (?1)",
                             params![task_name])
                    .expect("could not Insert");
                tasks.push(Tasks{
                    id:0,
                    name:String::from(task_name)
                });
                print_tasks(tasks);
            }else {
                println!("Task Name Needed");
            }
        },
        _ => println!("Not A Valid Add Argument")
    }
}
fn print_tasks(tasks:&Vec<Tasks>){
    clearscreen::clear().expect("Could Not Clear Screen");
    for i in 0..tasks.len(){
        let curr_task = &tasks[i];
        println!("{}: {}",i+1,curr_task.name);
    }
}
fn create_database_con()-> Result<Connection>{
    let conn = Connection::open("tasks.db")?;
    conn.pragma_update(None, "foreign_keys", true)?;
    Ok(conn)
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
fn get_sub_tasks(conn:&Connection,tasks:&Vec<Tasks>,task_to_be_opened:i32)->Vec<SubTasks>{
    let curr_task = tasks.get((task_to_be_opened-1) as usize);
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
        print_sub_tasks(&sub_tasks);
        sub_tasks
    }else {
        println!("trying to open none existing Task");
        sub_tasks
    }
}
fn print_sub_tasks(sub_tasks: &Vec<SubTasks>){
    clearscreen::clear().expect("Could Not Clear Screen");
    for i in 0..sub_tasks.len()  {
        let current_sub_task = &sub_tasks[i];
        println!("{}: {} {} {}",i+1,current_sub_task.name,if current_sub_task.is_ios_done==1{"(IOS Done)"}else{""},if current_sub_task.is_android_done==1{"(Android Done)"}else{""})
    }
}

