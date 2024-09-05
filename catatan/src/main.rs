use chrono::Local;
use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::fs::{self};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;


//struct untuk file txt nya
#[derive(Debug, Clone)]
struct File {
    name: String,
    content: String,
}

//struct untuk folder
#[derive(Debug)]
struct Folder {
    name: String,
    files: Vec<File>,
    subfolder: Vec<Rc<RefCell<Folder>>>,
    parent: Option<Rc<RefCell<Folder>>>,
}

fn garis() {
    let mut a = 0;
    while a < 60 {
        print!("=");
        a += 1;
    }
    println!("");
}

fn single() {
    let mut a = 0;
    while a < 60 {
        print!("-");
        a += 1;
    }
    println!("");
}

impl Folder {
    //construktor folder
    fn new(name: String, parent: Option<Rc<RefCell<Folder>>>) -> Rc<RefCell<Folder>> {
        Rc::new(RefCell::new(Folder {
            name,
            files: Vec::new(),
            subfolder: Vec::new(),
            parent,
        }))
    }

    //fungsi untuk menyimpan file
    fn add_file(&mut self, file: File) {
        self.files.push(file);
    }

    //fungsi untuk menyimpan folder
    fn add_folder(&mut self, folder: Rc<RefCell<Folder>>) {
        self.subfolder.push(folder);
    }

    //fungsi untuk menampilkan semua content yang ada
    // fn _list_conten(&self) {
    //     println!("Folder: {}", self.name);
    //     println!("Files : ");
    //     for file in &self.files {
    //         println!(" - {}", file.name);
    //     }

    //     println!("Subfolder : ");
    //     for folder in &self.subfolder {
    //         println!(" - {}", folder.name);
    //     }
    // }

    
    fn find_subfolder_by_name(&self, target_name: &str) -> Option<Rc<RefCell<Folder>>> { 
        // cek apakah nama folder sama 
        if self.name == target_name {
            return Some(Rc::new(RefCell::new(Folder{
                name: self.name.clone(),
                files: self.files.clone(),
                subfolder: self.subfolder.clone(),
                parent: self.parent.clone(),
            })));
        }
    
        // melakukan pencarian setiap subfolder
        for subfolder in &self.subfolder{
            let subfolder_ref: &RefCell<Folder> = subfolder;
            if let Some(found)  = subfolder_ref.borrow().find_subfolder_by_name(target_name){
                return Some(found);
            }
        }
    
        // jika folder tidak ada/tidak ditemukan 
        None
    }

    //masih error
    fn find_file(&mut self, name: &str) -> Option<&mut File> {
        self.files.iter_mut().find(|file| file.name == name)
    }

    //fungsi untuk mendapatkan path sekarang
    fn get_current_path(&self) -> PathBuf {
        let mut current_path = PathBuf::new();

        //bikin path untuk folder saat ini 
        current_path.push(&self.name);

        //melakukan traversing ke parent folder kalau ada 
        let mut folder = self;
        while let Some(parent) = &folder.parent {
            //meminjam folder induk 
            let parent_ref: Ref<Folder> = parent.borrow();

            //menambahkan path folder induk ke path sekarang 
            let parent_path = parent_ref.get_current_path();
            current_path = parent_path.join(current_path);

            //melanjutkan ke folder induk 
            folder = &*parent_ref;
        }

        current_path
    }
}

//untuk memastikan root sudah ada dan mengubah ke path direktori tersebut
fn create_root_set_root() -> io::Result<()> {
    let folder_path = "root";
    if !Path::new(folder_path).exists() {
        fs::create_dir(folder_path)?;
    }
    Ok(())
}

fn create_edit_file(root: &mut Folder) -> io::Result<String> {
    //cek apakah folder root sudah dibuat atau belum pernah dibuat
    create_root_set_root()?;

    //mengambil data waktu sekarang secara real time
    let curr_date = Local::now();
    let date = curr_date.format("%D%M%Y %H%M%S").to_string();

    //membuat nama dan path file sesuai timestap
    let mut file_name = String::new();
    println!("masukkan nama file yang diingin kan : ");
    io::stdin()
        .read_line(&mut file_name)
        .expect("cannot read line");
    let file_name = file_name.trim().to_string();

    //nama file yang fix penggabungan antara nama dari user dengan current time
    let fix_name = format!("{}/{}", file_name, date);

    //menyimpan file sesuai penggunanya
    if root.find_file(&file_name).is_none() {
        println!("creating  a new file with name : {}", fix_name);
        let file_content = String::new();

        let new_file = File {
            name: fix_name.clone(),
            content: file_content.clone(),
        };

        root.add_file(new_file);

        //mendapatkan current path sekarang untuk lokasi penyimpanan file
        //kenapa harus mendapatkan current path agar tidak terjadi kesalahan path saat menyimpan file
        let current_path = root.get_current_path();
        let path_file = current_path.join(&fix_name);
        let mut file = fs::File::create(&path_file)?;
        writeln!(file, "{}", file_content)?;

        //membuka file dengan teks editor yaitu notepad
        Command::new("Notepad")
            .arg(&path_file)
            .status()
            .expect("failed to open file with editor");

        Ok(file_name)
    } else {
        println!("file already exist with name : {}", file_name);
        Ok(file_name)
    }
}

fn create_folder(root: &Rc<RefCell<Folder>>) -> io::Result<String> {
    //cek apakah folder root sudah ada atau belum
    create_root_set_root()?;

    println!("masukkan nama folder: ");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("cannot read line");
    let folder_name = input.trim().to_string();

    //mendapatkan referensi mutable ke struct folder
    //karena kita menggunakan Rc<RefCell<T>> maka kita perlu memiliki akses mutable ke nilai yang dibungkus
    //oleh Rc<RefCell<T>>
    let mut root_ref = root.borrow_mut();
    
    if root_ref
        .borrow()
        .find_subfolder_by_name(&folder_name)
        .is_none()
    {
        //membuat folder di system
        let folder_path = root_ref.borrow().get_current_path().join(&folder_name);
        println!("creating folder at {}", folder_path.display());
        fs::create_dir(folder_path)?;

        //membuat folder di dalam struktur data
        let new_folder = Folder::new(folder_name.clone(), Some(Rc::clone(root)));
        root_ref.add_folder(new_folder);
        println!("folder {} berhasil dibuat.", folder_name);
        Ok(folder_name)
    } else {
        println!("folder dengan nama {} sudah ada njing.", folder_name);
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "folder already ada blog",
        ))
    }
}

//masih error 
fn list_root(root: &Folder) -> io::Result<()> {
    //membuat vector untuk menyimpan isi dari folder root
    let mut content = Vec::new();

    //melakukan loop untuk memasukkan ke dalam vector yang sudah di buat tadi
    for folder in &root.subfolder {
        content.push(format!("[Folder] {}", folder.name));
    }

    for file in &root.files {
        content.push(format!("[file] {}", file.name));
    }

    if content.is_empty() {
        println!("belum ada file atau folder yang dibuat ");
        return Ok(());
    }

    println!("Daftar isi root : ");
    for (index, contents) in content.iter().enumerate() {
        println!("{}, {}", index + 1, contents);
    }

    println!(
        "Pilih nomor yang ingin akses
tekan 0 untuk kembali"
    );

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("cannot read line");
    let pilihan = input.trim().parse::<usize>().unwrap_or(0);

    if pilihan == 0 {
        println!("kembali ke menu utama");
        return Ok(());
    } else if pilihan > 0 && pilihan <= content.len() {
        let selected = &content[pilihan - 1];
        if selected.starts_with("[Folder]") {
            let folder_name = selected.trim_start_matches("[Folder]").to_string();
            if let Some(selected_folder) = root.subfolder.iter().find(|f| f.name == folder_name) {
                // menampilkan isi folder jika folder dipilih
                // list_root_contents(selected_folder)?;
                list_root(selected_folder)?;
            }
        } else if selected.starts_with("[File]") {
            let file_name = selected.trim_start_matches("[File]").to_string();
            if let Some(selected_file) = root.files.iter().find(|f| f.name == file_name) {
                println!("membuka file {}", selected_file.name);

                Command::new("Notepad")
                    .arg(&selected_file.name)
                    .status()
                    .expect("cannot open the file with notepad");
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut root = Folder {
        name: "root".to_string(),
        files: Vec::new(),
        subfolder: Vec::new(),
        parent : None,
    };

    println!("Program Catatan");
    garis();
    println!("Menu utama");
    single();
    println!("[-] Catatan \n[-] Pengingat \n[-] Edit label \n[-] Keluar");
    single();
    println!("input : ");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("cannot read line");
    let input = input.trim();
    match input {
        "catatan" => {
            loop {
                single();
                println!("Buat catatan \nLihat catatan \nEdit catatan \nHapus catatan \nKembali");
                single();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("cannot read line");
                let input = input.trim();
                match input {
                    "buat" => {
                        println!("Folder baru");
                        println!("File baru");
                        println!("Folder yang sudah ada");
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).expect("cannot read line");
                        let input = input.trim();

                        match input {
                            "folder baru" => {
                                let new_folder_root = create_folder(&mut root)?;

                                if let Some(new_folder) = root.find_subfolder(&new_folder_root) {
                                    loop {
                                        println!("anda berada di folder {}", new_folder_root);
                                        println!("pilih aksi : ");
                                        println!("(1) Tambah File");
                                        println!("(2) Tambah Folder");
                                        println!("(3) kembali");
                                        single();
                                        println!("input : ");
                                        let mut input = String::new();
                                        io::stdin()
                                            .read_line(&mut input)
                                            .expect("cannot read line");
                                        let input = input.trim();

                                        match input {
                                            "1" => {
                                                //menambah file ke dalam folder
                                                create_edit_file(new_folder)?;
                                            }

                                            "2" => {
                                                create_folder(new_folder)?;
                                            }

                                            "3" => {
                                                break;
                                            }

                                            _ => println!("ra ono pilihan kui tol kontol"),
                                        }
                                    }
                                } else {
                                    println!("capek suu golei dewe ah cok");
                                }
                            }

                            _ => println!("ra ono pilihan kui anjingg"),
                        }
                    }

                    _ => println!("ra ono pilihan anjgg"),
                }
            }
        }

        _ => println!("ngga ada pilihan itu anjingg"),
    }

    Ok(())
}
