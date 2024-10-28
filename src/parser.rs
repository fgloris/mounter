use std::{path::PathBuf, fmt, process::Command, env, fs::OpenOptions, io::Read};

pub enum ErrType {
    IOError,
    LsblkErr,
    ReadLsblkErr,
    MountErr,
}

struct BlockInfo{
    name: String,
    uuid: String
}

struct MountBlockInfo{
    uuid: String,
    path: PathBuf
}

pub struct DiskInfo{
    force: bool,
    blocks: Vec<BlockInfo>,
    mount_list: Vec<MountBlockInfo>,
}

impl fmt::Debug for DiskInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        let mut debug_struct = f.debug_struct("DiskInfo");
        for block in &self.blocks{
            debug_struct.field(&block.name,&block.uuid);
        }
        for block in &self.mount_list{
            debug_struct.field(&block.uuid,&block.path);
        }
        debug_struct.finish()
    }
}

impl DiskInfo {
    fn find_name(&self, uuid: &String) -> Option<String>{
        for block in &self.blocks{
            if &block.uuid == uuid{
                return Some(block.name.clone());
            }
        }
        return None;
    }
    fn read_cfg(&mut self, value: String){
        let lines: Vec<&str> = value.split("\n").collect();
        let mut res : Vec<MountBlockInfo> = vec![];
        for line in lines{
            if line.len() == 0 {continue;}
            let words: Vec<&str> = line.trim().split(" ").collect();
            if words.len() != 2 {continue;}
            let uuid = words[0];
            let path = words[1];
            if uuid == "force" {
                self.force = match path {
                    "1"=>true,
                    "True"=>true,
                    "true"=>true,
                    "0"=>false,
                    "False"=>false,
                    "false"=>false,
                    _=>false   
                }
            }
            else {
                res.push(MountBlockInfo {uuid:uuid.to_string(),path:PathBuf::from(path)});
            }
        }
        self.mount_list = res;
    }
    fn read_lsblk(&mut self, value: String){
        let lines: Vec<&str> = value.split("\n").collect();
        let mut res : Vec<BlockInfo> = vec![];
        for line in lines{
            if line.len() == 0 {continue;}
            let words: Vec<&str> = line.trim().split(" ").collect();
            if words.len() < 2 {continue;}
            if words[0].contains("NAME") {continue;}
            let uuid = words[words.len()-1];
            let name = words[0];
            res.push(BlockInfo {name:name.to_string(),uuid:uuid.to_string()});
        }
        self.blocks = res;
    }
    //该程序在启动时执行,不能抛出运行时错误否则无法开机
    //force=true:中断并获取用户输入并再次尝试构建
    //force=false:放弃插入磁盘
    pub fn force(e: ErrType) -> Self{
        match e {
            ErrType::IOError=>{

            },
            ErrType::LsblkErr=>{

            },
            ErrType::ReadLsblkErr=>{

            },
            ErrType::MountErr=>{

            }
        }
        return DiskInfo{force: true, blocks:vec![],mount_list:vec![]};
    }
    pub fn new(force: bool) -> Option<Self>{
        let mut res = DiskInfo{force: force, blocks:vec![],mount_list:vec![]};
        if let Ok(mut path) = env::current_dir(){
            path.push("mount_info.cfg");
            if let Ok(mut file) = OpenOptions::new().read(true).open(path){
                let mut cfg = String::new();
                if let Ok(..) =  file.read_to_string(&mut cfg){
                    res.read_cfg(cfg);
                }else{  println!("cannot read config file!");       if res.force {return Some(DiskInfo::force(ErrType::IOError));}else{return None;}}
            }else{      println!("cannot open cfg file!\n");        if res.force {return Some(DiskInfo::force(ErrType::IOError));}else{return None;}}
        }else{          println!("cannot get current work dir!\n"); if res.force {return Some(DiskInfo::force(ErrType::IOError));}else{return None;}}

        let mut input = Command::new("sh");
        input.arg("-c").arg("lsblk -l -o NAME,UUID");
        if let Ok(out) = input.output() {
            if let Ok(lsblk) = String::from_utf8(out.stdout){
                res.read_lsblk(lsblk);
                return Some(res);
            }else{  println!("read lsblk output raised error!");if res.force {return Some(DiskInfo::force(ErrType::LsblkErr));}else{return None;}}
        }else{      println!("lablk command raised error!");    if res.force {return Some(DiskInfo::force(ErrType::ReadLsblkErr));}else{return None;}}
        
        
    }
    pub fn mount(&self){
        for mount in &self.mount_list {
            let path = &mount.path;
            let uuid = &mount.uuid;
            if let Some(name) = self.find_name(uuid) {
                if let Ok(r) = Command::new("sudo").arg("mount").arg(format!("/dev/{}",name)).arg(path.to_str().unwrap()).output(){
                    if r.status.success() {
                        println!("command success!");
                    }else{println!("mount command raised error: {}",String::from_utf8(r.stderr).unwrap());  if self.force {DiskInfo::force(ErrType::MountErr);}else{return;}}
                }else{    println!("mount command raised error!");                                          if self.force {DiskInfo::force(ErrType::MountErr);}else{return;}}
            }
        }
    }
}