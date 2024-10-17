pub const GO: &str = r#"
package main 

import "fmt"

func main() {
    fmt.Println("Hello, World from GO!")
}
"#;

pub const PYTHON: &str = r#"
print("Hello, World from PYTHON!")
"#;

pub const RUST: &str = r#"
fn main() {
    println!("Hello, World from RUST!");
}
"#;

pub const NODE: &str = r#"
console.log("Hello, World from NODE!");
"#;

pub const BASH: &str = r#"
echo "Hello, World from BASH!"
"#;

pub const PERL: &str = r#"
print "Hello, World from PERL!\n";
"#;

pub const RUBY: &str = r#"
puts "Hello, World from RUBY!"
"#;

pub const C: &str = r#"
#include <stdio.h>

int main() {
    printf("Hello, World from C!\n");
    return 0;
}
"#;

pub const CPP: &str = r#"
#include <iostream>

int main() {
    std::cout << "Hello, World from CPP!" << std::endl;
    return 0;
}
"#;

pub const PHP: &str = r#"
<?php

echo "Hello, World from PHP!\n";

?>
"#;

pub const JAVA: &str = r#"
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World from JAVA!");
    }
}
"#;

pub const SWIFT: &str = r#"
print("Hello, World from SWIFT!")
"#;

pub const SCALA: &str = r#"
object Main {
    def main(args: Array[String]): Unit = {
        println("Hello, World from SCALA!")
    }
}
"#;

pub const CSHARP: &str = r#"
using System;

class Program
{
    static void Main()
    {
        Console.WriteLine("Hello, World from CSHARP!");
    }
}
"#;
