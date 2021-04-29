use inflector::cases::*;
use pest::{iterators::Pair, Parser};
use std::{
    fmt::Debug,
    fs,
    io::Write,
};

#[derive(Parser)]
#[grammar = "ftd_compiler.pest"]
pub struct CSVParser;

#[derive(Debug, Clone, Default)]
pub struct StyleLines {
    pub style_name: String,
    pub style_value: String,
}

#[derive(Debug, Clone, Default)]
pub struct ParameterLines {
    pub parameter_name: String,
    pub parameter_value: String,
}

#[derive(Debug, Clone)]
pub enum FifthtryComponentsList {
    ROW,
    COLUMN,
}

impl Default for FifthtryComponentsList {
    fn default() -> Self {
        FifthtryComponentsList::ROW
    }
}

impl ToString for FifthtryComponentsList {
    fn to_string(&self) -> String {
        match self {
            FifthtryComponentsList::ROW => "E.row".into(),
            FifthtryComponentsList::COLUMN => "E.column".into(),
        }
    }
}

/*impl Debug for FifthtryComponentsList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FifthtryComponentsList::ROW => {
                write!(f, "{:?}", "row")
            },
            FifthtryComponentsList::COLUMN => {
                write!(f, "{:?}", "column")
            }
        }
    }
}*/
#[derive(Debug, Clone, Default)]
pub struct DeclarationLine {
    pub align: FifthtryComponentsList,
    pub childclass: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Record {
    pub declaration_line: DeclarationLine,
    pub id_line: String,
    pub parameter_lines: Vec<ParameterLines>,
    pub style_lines: Vec<StyleLines>,
}

#[derive(Debug, Clone, Default)]
pub struct Main {
    pub record: Record,
    pub child_record: Vec<Record>,
}

#[derive(Debug, Clone, Default)]
pub struct File {
    pub main: Vec<Main>,
}

fn main() {
    let unparsed_file = fs::read_to_string("Lib/try.ftd").expect("cannot read file");
    //    fs::write("foo.txt", b"Lorem ipsum")?;
    let file = CSVParser::parse(Rule::file, &unparsed_file)
        .expect("unsuccessful parse") // unwrap the parse result
        .next()
        .unwrap(); // get and unwrap the `file` rule; never fails

    let tokens = CSVParser::parse(Rule::file, &unparsed_file)
        .unwrap()
        .tokens();

    for token in tokens {
        println!("{:?}", token);
    }
    //    let mut record_count: u64 = 0;
    /*for file in file.clone().into_inner() {
        match file.as_rule() {
            Rule::main => {
                record_count += 1;
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }*/
    let mut file_struct = File::default();
    for file in file.into_inner() {
        match file.as_rule() {
            Rule::main => {
                let mut mains = Main::default();
                for main in file.into_inner() {
                    match main.as_rule() {
                        Rule::record => {
                            let mut records = Record::default();
                            creating_struct_process(main, &mut records);
                            mains.record = records;
                        }
                        Rule::child_record => {
                            let mut child_records = Record::default();
                            creating_struct_process(main, &mut child_records);
                            mains.child_record.push(child_records);
                        }
                        _ => unreachable!(),
                    }
                }
                file_struct.main.push(mains);
                //                record_count += 1;
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    println!("file_struct {:#?}", file_struct);
    let content = write_process(file_struct);
    println!("{}", content);

    let mut f = fs::File::create("Lib/try.elm").expect("Unable to create file");
    f.write_all(content.as_bytes())
        .expect("Unable to write into file");
    //    println!("Number of records: {}", record_count);
}

fn creating_struct_process(pair: Pair<Rule>, record_struct: &mut Record) -> () {
    match pair.as_rule() {
        Rule::fifthtry_row => {
            record_struct.declaration_line.align = FifthtryComponentsList::ROW;
            //            str::parse(pair.as_str()).unwrap()
        }
        Rule::fifthtry_column => {
            record_struct.declaration_line.align = FifthtryComponentsList::COLUMN;
        }
        Rule::childname => {
            record_struct
                .declaration_line
                .childclass
                .push(pair.as_str().to_string());
        }
        Rule::id_value => {
            record_struct.id_line = pair.as_str().to_string();
        }
        Rule::parameter_lines => {
            let mut parameter_lines = ParameterLines::default();
            for pairs in pair.into_inner() {
                match pairs.as_rule() {
                    Rule::parameter_name => {
                        parameter_lines.parameter_name = pairs.as_str().to_string();
                    }
                    Rule::parameter_value => {
                        parameter_lines.parameter_value = pairs.as_str().to_string();
                    }
                    _ => unreachable!(),
                }
            }
            record_struct.parameter_lines.push(parameter_lines);
        }
        Rule::style_lines => {
            let mut style_lines = StyleLines::default();
            for pairs in pair.into_inner() {
                match pairs.as_rule() {
                    Rule::style_name => {
                        style_lines.style_name = pairs.as_str().to_string();
                    }
                    Rule::style_value => {
                        style_lines.style_value = pairs.as_str().to_string();
                    }
                    _ => unreachable!(),
                }
            }
            record_struct.style_lines.push(style_lines);
        }
        Rule::EOI => (),
        _ => {
            for pairs in pair.into_inner() {
                creating_struct_process(pairs, record_struct)
            }
        }
    }
}

fn write_process(file: File) -> String {
    let mut write_string = String::from("");
    write_string.push_str("import Element as E\n");
    write_string.push_str("import Element.Background as Bg\n");
    write_string.push_str("import Element.Border as EB\n");
    write_string.push_str("import Element.Font as EF\n");
    write_string.push_str("import Element.Input as EI\n");
    write_string.push_str("import F\n");
    write_string.push_str("import Html as H\n");
    write_string.push_str("\n\n");

    for main in file.main {
        let mut declaration = format!(
            "{} : ",
            camelcase::to_camel_case(main.record.id_line.as_str())
        );
        let mut value = format!(
            "{} ",
            camelcase::to_camel_case(main.record.id_line.as_str())
        );
        for parameter_lines in main.record.parameter_lines {
            declaration.push_str(format!("{} -> ", parameter_lines.parameter_value).as_str());
            value.push_str(format!("{} ", parameter_lines.parameter_name).as_str());
        }
        declaration.push_str("F.Element msg");
        value.push_str("=");
        write_string.push_str(format!("{}\n", declaration).as_str());
        write_string.push_str(format!("{}\n", value).as_str());

        let mut elements = format!("\tF.e {} [", main.record.declaration_line.align.to_string());
        for i in 0..main.record.style_lines.len() {
            if i != 0 {
                elements.push_str(", ");
            }
            elements.push_str(
                format!(
                    "E.{} E.{}",
                    main.record.style_lines[i].style_name, main.record.style_lines[i].style_value
                )
                .as_str(),
            );
        }
        elements.push_str("] [");
        for i in 0..main.record.declaration_line.childclass.len() {
            if i != 0 {
                elements.push_str(", ");
            }
            elements.push_str(
                format!(
                    "{}",
                    camelcase::to_camel_case(main.record.declaration_line.childclass[i].as_str())
                )
                .as_str(),
            );
        }
        elements.push_str("]\n");
        write_string.push_str(elements.as_str());
        write_string.push_str("\n\n")
    }
    write_string
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test1() {
        main()
    }
}
