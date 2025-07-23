/* /packageApi
  Provides a set of functions for working with packages locally and remotely
  todo: Переделать if -> match
*/

/*
use std::{
  fs::{self, ReadDir, DirEntry, Metadata},
  io::{self, Write},
  path::{Path, PathBuf},
};

use crate::{log, logExit};
*/

//use reqwest::Error;
//use serde_json::Value;

/*
async fn fetchPackage(packageId: &str) -> Result<Value, Error> {
  let url = format!("https://realtime.su/api/packages/{}", packageId);
  let response = reqwest::get(&url).await?;
  let package = response.json::<Value>().await?;
  Ok(package)
}
// install package
async fn packageInstall(values: &Vec<String>) -> () {
  log("ok", &format!("Installing packages {:?}", values));

  for name in values {
    match fetchPackage(name).await {
      Ok(package) => {
        log("ok", &format!("Fetched package for {}: {}", name, package));

        if let Some(pkgName) = package.get("name") {
          log("ok", &format!("Package name: {}", pkgName));
        }
      }
      Err(err) => {
        log("error", &format!("Error fetching package {}: {}", name, err));
      }
    }
  }
}
*/

// основная функция для обращения ко всем остальным
/*
pub async fn packageApi(values: &Vec<String>, valuesLength: usize) -> () 
{ // check values length
  let valuesLength: usize = values.len();
  if valuesLength == 0 || values[0].len() == 0 { help() }
  //
  let mut error: bool = false;

  let _type: &String = &values[0];
  if _type == "help" { help(); } else
  if _type == "local" {
    if valuesLength >= 2 
    { // local <package name>
      if packageLocal(&values[1]) != 0 { error = true; }
    } else 
    { // local
      if packageLocal(".") != 0 { error = true; }
    }
  } else
  if _type == "local-delete" {
    if valuesLength >= 2 
    { // local-delete <project name>
      if packageLocalDelete(&values[1]) != 0 { error = true; }
    } else 
    { // local-delete
      if packageLocalDelete(".") != 0 { error = true; }
    }
  } else
  if _type == "push" && valuesLength >= 2 {
    println!("push");
  } else {
    error = true;
  }

  if error { logExit(1); }
}

// help
fn help() -> () 
{
  // todo: description
  log("ok","<empty>");
  log("ok","help");
  log("ok","local");
  log("ok","local-delete");
  log("ok","upload");
  log("ok","install");
  log("ok","uninstall");
  logExit(0);
}

// check that the directory already exists
fn checkDirectoryExist(packageName: &str) -> usize 
{
  if Path::new(packageName).exists() { return 1; }
  return 0;
}
// check if the JSON file already exists
fn checkJsonExist(path: &str) -> usize 
{
  if Path::new( &format!("{}{}",path,"package.json") ).exists() { return 1; }
  return 0;
}

// delete all files in directory
fn deleteAllFilesInDirectory(packagePath: &str) -> usize {
  return 
    match fs::read_dir(&packagePath) 
    {
      Ok(entries) => 
      {
        for entry in entries 
        {
          if let Ok(entry) = entry 
          {
            let path: &Path = &entry.path();
            // delete file || directory
            if path.is_dir() { let _ = fs::remove_dir_all(path); } // skip errors
            else             { let _ = fs::remove_file(path); }    // skip errors
          }
        }
        log("ok", &format!("All files in [{}] have been deleted", packagePath));
        0
      }
      Err(e) => 
      {
        log("err", &format!("Error reading directory [{}]: [{}]", packagePath, e));
        1
      }
    }
}

// create new package
fn createPackage(packageName: &str, inDirectory: bool) -> usize 
{ // create directory
  if !inDirectory {
    match fs::create_dir_all(packageName) {
      Ok(_) => log("ok", &format!("Directory [{}] created successfully", packageName)),
      Err(e) => {
        println!("Error creating directory [{}]: [{}]", packageName, e);
        return 1;
      }
    }
  }
  // create main.rt file
  let mainFilePath: &Path = &Path::new(packageName).join("main.rt");
  match fs::File::create(mainFilePath) {
    Ok(_) => log("ok", &format!("File [main.rt] created successfully")),
    Err(e) => {
      log("err", &format!("Error creating file [main.rt]: {}", e));
      return 1;
    }
  }
  // create JSON file
  let packageJsonPath: &Path = &Path::new(packageName).join("package.json");
  let mut packageJsonFile = match fs::File::create(packageJsonPath) {
    Ok(file) => file,
    Err(e) => {
      log("err", &format!("Error creating file [package.json]: {}", e));
      return 1;
    }
  };
  // json content
  let packageJsonContent: String = format!(
    r#"{{
  "name": "{}",
  "version": "1.0.0",
  "dependencies": {{}}
}}"#,
    if inDirectory 
    { 
      std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_default()
    } else { packageName.to_string()}
  );
  // write json content
  if let Err(e) = packageJsonFile.write_all(packageJsonContent.as_bytes()) {
    log("err", &format!("Error writing to [package.json]: {}", e));
    return 1;
  } else {
    log("ok", "File [package.json] created successfully");
  }
  return 0;
}
// create new local project
// e:
//   package local
//   package local <package name>
fn packageLocal(packageName: &str) -> usize 
{
  let inDirectory: bool = packageName == ".";
  if inDirectory 
  { // in directory mode [package local]
    // check files
    let currentDirectory: PathBuf = std::env::current_dir().expect("Failed to get current directory");
    let entries: ReadDir = fs::read_dir(currentDirectory).expect("Failed to read directory");
    // check if there are any files in the directory
    let hasFiles: bool = 
      entries
        .filter_map(Result::ok) // Convert Result<DirEntry, _> to Option<DirEntry>
        .any(|entry: DirEntry| {
          let metadata: Metadata = entry.metadata().expect("Failed to get metadata");
          metadata.is_file() || metadata.is_dir()
        });
    if hasFiles 
    {
      log("warn-input", "When creating the package in this directory, all files will be deleted. Proceed? (y/n): ");
      loop 
      { // get user input
        let mut input: String = String::new();
        io::stdout().flush().unwrap(); // ensure the prompt is printed before waiting for input
        io::stdin().read_line(&mut input).expect("Failed to read line");

        let input: &str = input.trim(); // remove whitespace
        match input 
        {
          "y" => { break; }    // exit the loop
          "n" => { return 1; } // exit
          _ => {
            log("warn-input", "Please enter [y] or [n]: ");
            io::stdout().flush().unwrap(); // ensure the prompt is printed before waiting for input
          }
        }
      }
      // remove all files in directory
      deleteAllFilesInDirectory(".");
    } else 
    { 
      log("ok", "No files were found in this folder when creating the package");
    }
    // create new package
    if createPackage(".",true) != 0 { return 1; }
  } else 
  { // not in directory mode [package local <package name>]
    // check directory exist
    if checkDirectoryExist(packageName) != 0 { 
      log("err", &format!("Package [{}] already exists", packageName));
      return 1; 
    }
    // check package JSON exist
    if checkJsonExist("") != 0 { 
      log("err", "You cannot create a new package in a directory with an existing package.json");
      return 1; 
    }
    // create new package
    if createPackage(packageName,false) != 0 { return 1; }
  }
  return 0;
}

// delete local package
// e:
//   package local-delete
//   pacakge local-delete <package name>
fn packageLocalDelete(packageName: &str) -> usize 
{
  let inDirectory: bool = packageName == ".";

  // check directory
  if !inDirectory 
  {
    if checkDirectoryExist(packageName) != 1 
    { 
      log("err", &format!("Package [{}] does not exist", packageName));
      return 1; 
    }
  }
  // check package JSON
  if inDirectory 
  {
    if checkJsonExist("") != 1 
    {
      log("err", "You can't delete a package without package.json inside");
      return 1; 
    }
  } else 
  {
    if checkJsonExist(&format!("{}/",packageName)) != 1 
    {
      log("err", "You can't delete a package without package.json inside");
      return 1; 
    }
  }

  return 
    if inDirectory
    { // delete all files in directory
      deleteAllFilesInDirectory(".")
    } else 
    { // delete package directory and files
      match fs::remove_dir_all(Path::new(packageName)) 
      {
        Ok(_) => 
        {
          log("ok", &format!("Package [{}] and its contents have been removed successfully", packageName));
          0
        }
        Err(e) => 
        {
          log("err", &format!("Error removing directory [{}]: [{}]", packageName, e));
          1
        }
      }
    }
}
*/