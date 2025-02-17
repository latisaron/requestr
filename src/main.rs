use clap::Parser;
use reqwest;
use tokio::task;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use std::sync::Arc;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    link: String,
    #[arg(short, long, default_value_t = 5)]
    workers: i32,
    #[arg(short, long)]
    requests: i32,
    #[arg(short, long, default_value_t = false)]
    analysis: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let mut handles = vec![];
    
    let start_time = Instant::now();

    let status_counts = Arc::new(Mutex::new(HashMap::<i16, i32>::new()));
    let response_times = Arc::new(Mutex::new(Vec::<u128>::new()));

    let mut current_request_number = 0;
    let mut loop_flag = true;

    while loop_flag {
        for _ in 0..args.workers {
            current_request_number += 1;
            let url_clone = args.link.to_string();
            let status_counts_clone = Arc::clone(&status_counts);
            let response_times_clone = Arc::clone(&response_times);
    
            handles.push(task::spawn(async move {
                match reqwest::get(&url_clone).await {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        
                        let mut map = status_counts_clone.lock().unwrap();
                        *map.entry(status as i16).or_insert(0) += 1;

                        let mut rt_vec = response_times_clone.lock().unwrap();
                        rt_vec.push(start_time.elapsed().as_millis());

                    }
                    Err(e) => {
                        let mut map = status_counts_clone.lock().unwrap();
                        *map.entry(-1).or_insert(0) += 1;
                    }
                }
            }));
            if current_request_number == args.requests {
                loop_flag = false;
                break;
            }
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("All requests completed in {:?}", start_time.elapsed());
    println!("The statuses of the requests are: {:?}",  status_counts.lock().unwrap());
    println!("Response times are {:?}", response_times.lock().unwrap());

}
