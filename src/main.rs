use actix_web::{get,web::Data, App, HttpResponse, HttpServer, Responder};
use std::{fs, sync::Arc};
use std::path::PathBuf;
use std::process::Command;
use rand::{Rng, thread_rng};


/// bash.sh in ./images
/// ```
/// #!/bin/sh
/// ls -l | grep "^-" | wc -l
/// ```
/// `ls -l`       get all file's info
/// 
/// `grep "^-"`   filter folders
/// 
/// `wc -l`       count the number of files
/// 
/// but `bash.sh` also be counted,
/// so the real picture number need to -1
/// 
async fn get_how_many_pictures_in_images()->i128{
    let number = Command::new("sh")
        .current_dir("./images")
        .arg("-C")
        .arg("bash.sh")
        .output()
        .expect("failed to run number sh");
    let number = String::from_utf8_lossy(&number.stdout)
        .into_owned()
        .trim()
        .parse::<i128>()
        .expect("failed to prase number string to i128");
    return number-1;
}

/// # Example
/// ```
///     let picture = get_random_picture_name();
///     assest_eq!((picture),"1.png");
/// ```
async fn get_random_picture_name()->String{
    let number =get_how_many_pictures_in_images().await;
    let mut random_num:i128 =-1;
    match number {
        0 => panic!("the images is empty"),
        _ => {
            let mut rng = thread_rng();
            random_num = rng.gen_range(1..=number);
        }
    }
    let picture = random_num.to_string()+".png";
    return picture;
}


async fn get_picture_base64()->String{
    let picture = get_random_picture_name().await;
    // get the file path
    let mut image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    image_path.push("images");
    image_path.push(picture);

    let image_path = image_path.to_str()
        .expect("image_path get error");
    let base64 = image_base64::to_base64(image_path);
    return base64;
}


#[get("/")]
async fn random_img_base64_html(html_base:Data<Arc<String>>) -> impl Responder{
    let base64 = get_picture_base64().await;
    let html = html_base.replace("imgdata", &base64);
    HttpResponse::Ok().body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let html_base = Data::new(Arc::new(fs::read_to_string("u1.html").unwrap()));
    HttpServer::new(move ||{
        App::new()
        .app_data(Data::clone(&html_base))
        .service(random_img_base64_html)
    })
    .workers(16)
    .bind(("0.0.0.0",8080))?
    .run()
    .await
}
