pub const GO: &str = r#"
package main 

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
"#;

pub const PYTHON: &str = r#"
print("Hello, World!")
"#;

pub const RUST: &str = r#"
fn main() {
    println!("Hello, World!");
}
"#;

pub const NODE: &str = r#"
console.log("Hello, World!");
"#;

pub const BASH: &str = r#"
echo "Hello, World!"
"#;

pub const PERL: &str = r#"
print "Hello, World!\n";
"#;

pub const RUBY: &str = r#"
puts "Hello, World!"
"#;

pub const C: &str = r#"
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}

"#;

pub const CPP: &str = r#"
#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}

"#;

pub const PHP: &str = r#"
<?php

echo "Hello, World!\n";

?>

"#;

pub const JAVA: &str = r#"
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
"#;

pub const SWIFT: &str = r#"
print("Hello, World!")
"#;

pub const SCALA: &str = r#"
object Main {
    def main(args: Array[String]): Unit = {
        println("Hello, World!")
    }
}
"#;

pub const CSHARP: &str = r#"
using System;

class Program
{
    static void Main()
    {
        Console.WriteLine("Hello, World!");
    }
}

"#;
