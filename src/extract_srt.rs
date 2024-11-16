use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

pub enum Separator {
    NEWLINE,
    SPACE,
}

pub fn extract_to<P: AsRef<Path>>(input: P, output: P, sep: Separator) -> io::Result<()> {
    let input = input.as_ref();
    let output = output.as_ref();

    if output.exists() {
        println!("Output file already exists, Whether to overwrite?...");
        print!("Press Y to overwrite, press any other key to abort [Y/N]: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Output file already exists, Aborted by user",
            ));
        }
    }

    let input_file = File::open(input)?;
    let reader = BufReader::new(input_file);

    let output_file = OpenOptions::new().write(true).create(true).open(output)?;
    let mut writer = BufWriter::new(output_file);

    let mut flag = false;
    let mut line_count = 0;
    // 逐行读取文件
    for line in reader.lines() {
        let line = line?; // 解析行内容，如果读取错误则返回

        // 在这里进行逻辑处理，例如：只写入包含"特定内容"的行
        if line.contains(" --> ") {
            flag = true;
            continue;
        }

        if flag {
            // 如果条件满足，写入到新文件
            match sep {
                Separator::NEWLINE => writeln!(writer, "{}", line)?,
                Separator::SPACE => write!(writer, " {}", line)?,
            }
            line_count += 1;
            flag = false;
        }
    }

    writer.flush()?;
    println!("Extract {} lines successfully!", line_count);

    Ok(())
}
