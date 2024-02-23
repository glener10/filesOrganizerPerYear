use rand::Rng;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::directory::list_files_in_directory;
use crate::errors::AppError;
use crate::file::{get_file_extension, get_file_modification_date};
use crate::operation::FILE_OPERATION;
use crate::organize_logs::print_executing_log;

struct ExtensionCounter {
  extension: String,
  count: usize,
}

pub fn organize_files(dir_path: &str, output_directory: &str) -> Result<(), AppError> {
  let paths = list_files_in_directory(dir_path)?;
  let paths_len = paths.len();

  let mut count_files = 0;
  let mut count_files_with_same_name = 0;

  let mut files_transfered = HashSet::<String>::new();
  let mut files_with_repeat_name: Vec<String> = Vec::new();
  let mut extension_counters: Vec<ExtensionCounter> = Vec::new();

  println!("Organizing Files...");
  for path in paths {
    let file_extension = get_file_extension(&path.to_string_lossy())?;
    let mut extension_exists = false;

    for counter in &mut extension_counters {
      if counter.extension == file_extension {
        counter.count += 1;
        extension_exists = true;
        break;
      }
    }
    if !extension_exists {
      let new_counter = ExtensionCounter {
        extension: file_extension.clone(),
        count: 1,
      };
      extension_counters.push(new_counter);
    }

    let modification_year = get_file_modification_date(&path.to_string_lossy())?;

    let output_dir = PathBuf::from(output_directory).join(format!("{}", modification_year));
    fs::create_dir_all(&output_dir)?;

    let mut file_name = path.file_name().ok_or_else(|| {
      AppError::IO(std::io::Error::new(
        std::io::ErrorKind::Other,
        "No file name",
      ))
    })?;

    let file_name_str = file_name.to_string_lossy().to_string();
    let new_name_with_random_id: OsString;

    if !files_transfered.insert(file_name_str.clone()) {
      count_files_with_same_name += 1;
      let mut rng = rand::thread_rng();
      let random_id: u32 = rng.gen();
      let old_file_name = file_name_str.clone();
      new_name_with_random_id = OsString::from(format!("{}_{}", random_id, file_name_str));
      file_name = OsStr::new(&new_name_with_random_id);
      let old_and_new_file_name = format!(
        "Old File Name: {} - New File Name: {}",
        old_file_name,
        file_name.to_string_lossy()
      );
      files_with_repeat_name.push(old_and_new_file_name);
      files_transfered.insert(file_name.to_string_lossy().to_string());
    }

    let output_file = output_dir.join(file_name);

    unsafe {
      FILE_OPERATION
        .execute(&path.to_string_lossy(), &output_file.to_string_lossy())
        .unwrap();
    }
    count_files += 1;
    print_executing_log(count_files, paths_len, file_name);
  }
  println!("\nFinish!\n");

  create_repeated_name_log_file(
    count_files_with_same_name,
    files_with_repeat_name,
    output_directory,
  );

  if count_files > 0 {
    for counter in &extension_counters {
      println!(
        "Total of {} files with '.{}' extension",
        counter.count, counter.extension
      );
    }
    println!("\n\nTotal of {} files organized", count_files);
  }
  Ok(())
}

fn create_repeated_name_log_file(
  count_files_with_same_name: i32,
  files_with_repeat_name: Vec<String>,
  output_directory: &str,
) {
  if count_files_with_same_name > 0 {
    let output_file_path = format!("{}/filesWithRepeatedName.txt", output_directory);
    let output_file = File::create(&output_file_path);

    match output_file {
      Ok(output_file) => {
        for result in &files_with_repeat_name {
          let write_in_file = writeln!(&output_file, "{}", result);
          if write_in_file.is_err() {
            println!(
              "An error occurred when trying to write the result to the file '{}': {}",
              output_file_path, result
            );
          }
        }
        println!(
          "Total of {} file with the repeated name\n",
          count_files_with_same_name
        );
      }
      Err(err) => {
        println!(
          "An error occurred when trying to create the file: '{}'. Error: {}",
          output_directory, err
        );
      }
    }
  }
}
