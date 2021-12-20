/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

     https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use tree_sitter::{Language, Parser, Query, Tree};
use tree_sitter::{Node, TreeCursor};

#[macro_use]
extern crate log;

pub mod builder;
mod capture;
mod util;

#[cfg(feature = "python")]
pub mod python;
pub mod query;
pub mod result;

extern "C" {
    fn tree_sitter_c() -> Language;
    fn tree_sitter_cpp() -> Language;
}

/// Helper function to parse an input string
/// into a tree-sitter tree, using our own slightly modified
/// C grammar. This function won't fail but the returned
/// Tree might be invalid and contain errors.
pub fn parse(source: &str, cpp: bool) -> Tree {
    let language = if !cpp {
        unsafe { tree_sitter_c() }
    } else {
        unsafe { tree_sitter_cpp() }
    };
    let mut parser = Parser::new();
    if let Err(e) = parser.set_language(language) {
        eprintln!("{}", e);
        panic!();
    }

    parser.parse(source, None).unwrap()
}


pub fn find_node_by_type<'a>(node:tree_sitter::Node<'a>,kind:&'a str)->Node<'a> {
    let mut cursor = node.walk();
    loop {
        if cursor.node().kind() == kind {
            let node = cursor.node();
            break node;
        }
        if  !cursor.goto_first_child(){
            while !cursor.goto_next_sibling(){
                if !cursor.goto_parent(){
                    break;
                }
            }
        }
    }
}


pub fn find_node_by_field_and_get_content(node:tree_sitter::Node,name:& str,code:&str)->String {
    let cursor = node.walk();
    let mut result = String::new();
    if let Some(child)=  cursor.node().child_by_field_name(name){
        result+=&code[child.start_byte()..child.end_byte()];
    }
    result
}


// Internal helper function to create a new tree-sitter query.
fn ts_query(sexpr: &str, cpp: bool) -> tree_sitter::Query {
    let language = if !cpp {
        unsafe { tree_sitter_c() }
    } else {
        unsafe { tree_sitter_cpp() }
    };

    match Query::new(language, &sexpr) {
        Ok(q) => q,
        Err(e) => {
            eprintln!(
                "Tree sitter query generation failed: {:?}\n {}",
                e.kind, e.message
            );
            eprintln!("sexpr: {}", sexpr);
            eprintln!("This is a bug! Can't recover :/");
            std::process::exit(1);
        }
    }
}

