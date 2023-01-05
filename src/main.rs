use std::env;
use std::process;
use std::str;

//for file reading 
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

// threads
use std::{thread, time};
use std::time::Duration;

//for thread communication
use std::sync::mpsc;

// for enum conversion
extern crate strum;
#[macro_use]
extern crate strum_macros;

// for the type function (to check types)
use std::any::type_name;


use schedule_sim::Config;
//use schedule_sim::DoublyLinkedList;
use std::collections::LinkedList;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem passing arguments: {err}");
        process::exit(1);
    });

   #[derive(EnumString)]
    enum SchedulingPolicy {
        FCFS, 
        RR, 
        PRI, 
        SPN,
    }
    // if let Err(e) = schedule_sim::run(config) {
    //     println!("Application Error: {e}");
    //     process::exit(1);
    // }

    // configuration information passed from the user via command line
    let _algorithm = &config.algorithm.to_uppercase();
    let _file = &config.file_path;
    let _quantum = &config.quantum;
    

    println!("using alorithm {}", _algorithm);
    println!("On mock process file {}", _file);
    if _quantum != "0" {
        println!("quantum for round robin {}", _quantum);
    }
    println!("\n");

    fn read_file_to_vec(file: &str) -> Vec<String> {
        let _f = File::open(file).expect("error opening file");
        let buf = BufReader::new(_f);
        buf.lines()
            .map(|l| l.expect("could not parse line in file"))
            .collect()
    }

    // was previously implementing my own linked list to customize for atmoicty and concurrent access
    //let mut _ready_queue: DoublyLinkedList<String> = DoublyLinkedList::new();
    
    let mut _ready_queue = LinkedList::new();
    let mut _io_queue = LinkedList::new();

    // channel for concurrent thread communication
    let (tx, rx) = mpsc::channel();

    fn type_of<T>(_: T) -> &'static str {
        type_name::<T>()
    }

    let read_file = thread::spawn(move || {
        let lines = read_file_to_vec(&config.file_path);
        let _process_info: Vec<&str> = vec![""]; 
        for line in lines {
            //println!("{}", line);
            let service_info: Vec<&str> = line.split(" ").collect();
            for item in service_info {
                let mut process_info: Vec<&str> = Vec::new();
                process_info.push(item);

                for el in process_info {
                    tx.send(el.to_string()).unwrap();
                    //println!("{}", el);
                }     
            };
        }
    });
    read_file.join().unwrap();

    for received in rx {
        //println!("Got {}", received);
        _ready_queue.push_back(received.clone());
    }

    let mut ready_queue = _ready_queue.clone();
    let mut algorithm_ = _algorithm.clone();
    let mut quantum = _quantum.clone();
    let time_allocation: u64 = quantum.parse().unwrap_or(1); 

    // set scheduling policy based on user input, defaults to first come first serve
    let scheduling_algorithm: SchedulingPolicy = 
        if algorithm_ == "FCFS" {
            SchedulingPolicy::FCFS
        } else if algorithm_ == "RR" {
            SchedulingPolicy::RR
        } else if algorithm_ == "PRI" {
            SchedulingPolicy::PRI
        } else if algorithm_ == "SPN" {
            SchedulingPolicy::SPN
        } else {
            SchedulingPolicy::FCFS
        };
   

    //println!("{}", type_of(queue));
    let (cpu, io) = mpsc::channel();

    let simulate_cpu = thread::spawn(move || {
        println!("recieved ready queue");
        println!("using scheduling policy {}", algorithm_);
        thread::sleep(Duration::from_secs(2));

        if algorithm_ == "FCFS" {
            let fcfs = SchedulingPolicy::FCFS;
        } 
        // use the users selected algorithm - set this value on line 109
        match scheduling_algorithm {
            SchedulingPolicy::FCFS =>  { loop {
                let first_read = ready_queue.pop_front();
                let _converted_first = first_read.as_deref().unwrap_or("stop");
                //println!("process {}", _converted_first);
                if _converted_first == "proc" {
                    println!("starting process...");
                    thread::sleep(Duration::from_secs(1));
                    
                loop {
                        let second_read = ready_queue.pop_front();
                        let _converted_second = second_read.as_deref().unwrap_or("default");
                        let service_time: u64 = _converted_second.parse().unwrap_or(1); 
                        if service_time == 1 {
                            break;
                        }
                        //println!("first service time {}", service_time);
                        let _time = time::Duration::from_millis(service_time);
                        let now = time::Instant::now();
        
                        thread::sleep(_time);
        
                        println!("cpu in use {} milliseconds", service_time);
        
                        assert!(now.elapsed() >= _time);
    
                        let read_for_io = ready_queue.pop_front();
                        let _converted_for_io = read_for_io.as_deref().unwrap_or("default");
    
    
                        cpu.send(_converted_for_io.to_string());
                        thread::sleep(Duration::from_secs(1));
    
                    }
                        
            
                    } else if _converted_first == "sleep" {
                        println!("wait");
                        thread::sleep(Duration::from_secs(2));
                        let read_wait = ready_queue.pop_front();
                        let _converted_wait = read_wait.as_deref().unwrap_or("default");
                        let wait_time: u64 = _converted_wait.parse().unwrap_or(1); 
    
                        let _wait_time = time::Duration::from_millis(wait_time);
                        let now = time::Instant::now();
        
                        thread::sleep(_wait_time);
        
                        println!("cpu in idle {} milliseconds", wait_time);
        
                        assert!(now.elapsed() >= _wait_time);
    
                        continue;
                    } else if _converted_first == "stop" {
                        println!("quit");
                        break;
                    } 
                }
           }
           SchedulingPolicy::RR => {  loop {
                // convert the quantum into an integer
                //let time_allocation: u64 = quantum.parse().unwrap_or(1); 
                let first_read = ready_queue.pop_front();
                let _converted_first = first_read.as_deref().unwrap_or("stop");
                //println!("process {}", _converted_first);
                if _converted_first == "proc" {
                    println!("starting process...");
                    thread::sleep(Duration::from_secs(1));
                loop {
                        let second_read = ready_queue.pop_front();
                        let _converted_second = second_read.as_deref().unwrap_or("default");
                        let mut service_time: u64 = _converted_second.parse().unwrap_or(1); 
                        if service_time == 1 {
                            break;
                        }
                        //println!("first service time {}", service_time);
                        if service_time < time_allocation {
                            service_time %= time_allocation;
                            ready_queue.push_front(service_time.to_string());
                        } else {
                            service_time -= time_allocation;
                            ready_queue.push_front(service_time.to_string());
                        }
                        
                        let _time = time::Duration::from_millis(time_allocation);
                        let now = time::Instant::now();
        
                        thread::sleep(_time);
        
                        println!("cpu in use {} milliseconds", time_allocation);
                        let total_cpu_time = time_allocation + service_time;  
                        // println!("Analysis for Round Robin: ")
                        // println!("Analysis for Round Robin")
                        // println!("Analysis for Round Robin")
                        // println!("Analysis for Round Robin")

        
                        assert!(now.elapsed() >= _time);

                        let read_for_io = ready_queue.pop_front();
                        let _converted_for_io = read_for_io.as_deref().unwrap_or("default");


                        cpu.send(_converted_for_io.to_string());
                        thread::sleep(Duration::from_secs(1));
    
                    }
                } else if _converted_first == "wait" {
                    println!("wait");
                } else if _converted_first == "stop" {
                    println!("total cpu {}", time_allocation);
                    println!("quit");
                    break;
                }
                //println!("total cpu {}", time_allocation);

            }
            //println!("total cpu {}", time_allocation);
        }
        
           SchedulingPolicy::PRI => { println!("pri") }
           SchedulingPolicy::SPN => { println!("spn") }
        }
    });


    simulate_cpu.join().unwrap();

    for received_io in io {
        //println!("Got {} in I/O", received_io);
        _io_queue.push_back(received_io.clone());
    }

    let mut io_queue = _io_queue.clone();
    
   
    let simulate_io = thread::spawn (move || {
        println!("recieved i/o queue");
        
        //println!("using scheduling policy {}", algorithm_);

        loop {
            let read = io_queue.pop_front();
            let _converted_read = read.as_deref().unwrap_or("default");
            let service_time: u64 = _converted_read.parse().unwrap_or(1); 
            if service_time == 1 {
                break;
            }
            //println!("first service time {}", service_time);
            let _time = time::Duration::from_millis(service_time);
            let now = time::Instant::now();

            thread::sleep(_time);
            println!("i/o in use {} milliseconds", service_time);

            assert!(now.elapsed() >= _time);

        }

    });

    simulate_io.join().unwrap();

    // for el in _ready_queue {
    //     println!("{}", el);
    // }

}




