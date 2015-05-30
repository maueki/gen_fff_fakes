#![feature(libc)]
extern crate libc;

mod clang;
mod clangll;

use clang::{ast_dump, Cursor, Diagnostic, TranslationUnit};
use clangll::*;

use std::env;
use std::ffi::{CString, CStr};
use std::default;
use std::str;
use std::process::exit;

// String
struct String_ {
    x: CXString
}

impl String_ {
    fn to_string(&self) -> String {
        if self.x.data.is_null() {
            return "".to_string();
        }
        unsafe {
            let c_str = clang_getCString(self.x) as *const libc::c_char;
            let p = c_str as *const _;
            str::from_utf8(CStr::from_ptr(p).to_bytes()).unwrap().to_string()
        }
    }
}

fn type_to_str(ty: &clang::Type) -> String{
    let tystr = match ty.kind() {
        CXType_Void => "void".to_string(),
        CXType_Bool => "bool".to_string(),
        CXType_UInt => "unsigned int".to_string(),
        CXType_Int => "int".to_string(),
        CXType_UChar => "unsigned char".to_string(),
        CXType_Char_S => "char".to_string(),
        CXType_Float => "float".to_string(),
        CXType_Double => "double".to_string(),
        CXType_Long => "long".to_string(),
        CXType_Typedef => {
            type_to_str(&ty.declaration().typedef_type())
        },
        CXType_Pointer => {
            format!("{}*", type_to_str(&ty.pointee_type()))
        }
        _ => String_{ x: unsafe{ clang_getTypeKindSpelling(ty.kind()) } }.to_string()
    };

    if ty.is_const() {
        return format!("{} const",tystr);
    }

    tystr
}

pub fn gen_fff_fake(c: &Cursor, funcs: &mut Vec<String>)-> Enum_CXVisitorResult {
    match c.kind() {
        CXCursor_FunctionDecl => {
            match c.ret_type().kind() {
                CXType_Void => {
                    print!("FAKE_VOID_FUNC(");
                },
                _ => {
                    print!("FAKE_VALUE_FUNC({},", type_to_str(&c.ret_type()));
                }
            }
            print!("{}", c.spelling());
            funcs.push(c.spelling());
            let args: Vec<String> = c.cur_type().arg_types().iter().map(|arg| {
                type_to_str(arg)
            }).collect();
            for a in args.iter() {
                print!(", {}", a);
            }
            println!(");");
            CXChildVisit_Continue
        },
        _ => CXChildVisit_Continue
    }
}

fn parse(path: &str) {

    let mut ix = clang::Index::create(false, true);

    let tu = TranslationUnit::parse(&ix, path,
                                    &vec![], &vec![], 0);

    let cursor = tu.cursor();
    let mut funcs: Vec<String> = vec![];

    // generate fakes
    cursor.visit(|cur, _:&Cursor| gen_fff_fake(cur, &mut funcs));

    // generate fakes list (for RESET_FAKE)
    println!(r"#define FFF_FAKES_LIST(FAKE) \");
    for f in funcs.iter() {
        println!(r"FAKE({})\", f);
    }
    println!("");
}

fn usage() {
    println!("gen_fff_dummy FILE");
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        usage();
        exit(-1);
    }

    parse(&args[1]);
}
