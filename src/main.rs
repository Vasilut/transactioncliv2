use std::{env, error::Error, io};

pub mod entities;
pub mod service;
use entities::Transaction;
use service::PaymentEngine;

fn main() -> Result<(), Box<dyn Error>>{

    let args : Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("We need to pass only one parameter, this is how we need to use it: cargo run -- <transactions.csv>");
        return Err("Invalid number of arguments".into());
    }

    let input_file = &args[1];
    let mut file_reader = csv::Reader::from_path(input_file)?;

    let mut payment_engine = PaymentEngine::new();

    for records in file_reader.deserialize::<Transaction>() {
        match records {
            Ok(current_transaction) => {
                payment_engine.process_transaction(current_transaction);
            }
            Err(e) => {
                eprintln!("Error processing record: {}", e);
                continue; 
            }
        }
    }

    let accounts = payment_engine.get_client_accounts();

    let mut csv_writer = csv::Writer::from_writer(io::stdout());
    for account in accounts {
        csv_writer.serialize(account)?;
    }
    
    csv_writer.flush()?;

    Ok(())
}
