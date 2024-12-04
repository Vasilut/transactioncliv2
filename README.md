### Transaction CLI

A transaction CLI which can support different operation like Deposit, Withdrawal, Dispute, Resolve, Chargeback, etc.

This is a cargo rust project.

For installing rust please follow this: https://doc.rust-lang.org/beta/book/ch01-01-installation.html

To create a cargo project:

* ```cargo new transaction_cli```

To compile:

* ```cargo build```
* ```cargo check```


To run the program:

 ```cargo run -- transactions.csv > accounts.csv```

 Input file: (transactions.csv):

```
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
withdrawal,2,5,0.27
```

Output file: (accounts.csv)

```
client,available_amount,held_amount,total_amount,locked
1,1.5,0.0,1.5,false
2,1.73,0.0,1.73,false
```

To run the tests:

```cargo test```
