use actix_web::{get,web,App,Error, HttpResponse, HttpServer, Responder,web::Data};
use actix_multipart::Multipart;
use std::{fs, sync::Arc};

use std::process::Command;
use futures::TryStreamExt;
use uuid::Uuid;
use std::io::Write;
use rand::{Rng, thread_rng};
use std::collections::{HashMap, HashSet};

/// 用于post获得上传文件，上传文件存储在tmp文件夹中
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut res:HashSet<String> = HashSet::new();
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await?
        let file_num = get_how_many_pictures_in_images().await+1;
        let filename = file_num.to_string()+".png";
        let filepath = format!("/images/images/{}", filename);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await??;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
        {
            let url = "http://setu.works/target/".to_string()+file_num.to_string().as_str();
            res.insert(url);
        }
    }
    let result = serde_json::to_string(&res).unwrap();
    Ok(HttpResponse::Ok().body(result))
}

/// 用于post获得上传文件，上传文件存储在tmp文件夹中
async fn upload_zip(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), sanitize_filename::sanitize);
        let filepath = format!("./tmp/{}", filename);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath)).await??;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
        }
    }
    Ok(HttpResponse::Ok().body("success!"))
}

async fn upload_index(html_base:Data<Arc<HashMap<String, Data<Arc<String>>>>>) -> Result<HttpResponse, Error> {
    let html = html_base.get("upload").unwrap().to_string();
    Ok(HttpResponse::Ok().body(html))
}






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
    return number;
}

/// # Example
/// ```
///     let picture = get_random_picture_id();
///     assest_eq!((picture),"1.png");
/// ```
async fn get_random_picture_id()->String{
    let number =get_how_many_pictures_in_images().await;
    let mut random_num:i128 =-1;
    match number {
        0 => panic!("the images is empty"),
        _ => {
            // get random number
            let mut rng = thread_rng();
            random_num = rng.gen_range(1..=number);
        }
    }
    let picture_id = random_num.to_string();
    return picture_id;
}


async fn get_picture_url(picture_id : &String)->String{
    let picture_id = picture_id.to_owned()+".png";
    let url = "http://p.c76d.xyz/images/".to_string()+&picture_id;
    return url;
}

#[get("/target/{pictureid}")]
async fn target_img_html(picture_id : web::Path<i64>,html_base:Data<Arc<HashMap<String, Data<Arc<String>>>>>)-> HttpResponse{
    let exist_picture_number = get_how_many_pictures_in_images().await;
    let picture_id=picture_id
        .to_string()
        .trim()
        .parse::<i128>().unwrap();

    if (picture_id<=0)||(picture_id>exist_picture_number){
        let sorry = exist_picture_number.to_string()+" is alive,this picture do not exist";
        return HttpResponse::Ok().body(sorry);
    }
    
    let picture_id = picture_id.to_string();

    let url = get_picture_url(&picture_id).await;
    let html_base = html_base.get("random").unwrap();
    let html = html_replace(html_base, url, picture_id).await;
    HttpResponse::Ok().body(html)
}

#[get("/random")]
async fn random_img_html(html_base:Data<Arc<HashMap<String, Data<Arc<String>>>>>) -> impl Responder{
    let picture_id = get_random_picture_id().await;
    let url = get_picture_url(&picture_id).await;
    let html_base = html_base.get("random").unwrap();
    let html = html_replace(html_base, url, picture_id).await;
    HttpResponse::Ok().body(html)
}

#[get("/")]
async fn random(html_base:Data<Arc<HashMap<String, Data<Arc<String>>>>>) -> impl Responder{
    let picture_id = get_random_picture_id().await;
    let url = get_picture_url(&picture_id).await;
    let html_base = html_base.get("random").unwrap();
    let html = html_replace(html_base, url, picture_id).await;
    HttpResponse::Ok().body(html)
}

async fn html_replace(html_base:&Data<Arc<String>>,url:String,picture_id:String)->String{
    let html_base = html_base.replace("imgname",&(picture_id+".png"));
    let html = html_base.replace("imgdata", &url);
    return html;
}

/// 准备html文件
async fn html_base_make()->HashMap<String, Data<Arc<String>>>{
    let mut html_base = HashMap::new();
    let random_html_base = Data::new(Arc::new(fs::read_to_string("./html/random.html").unwrap()));
    let upload_html_base = Data::new(Arc::new(fs::read_to_string("./html/upload.html").unwrap()));
    html_base.insert(String::from("random"), random_html_base);
    html_base.insert(String::from("upload"), upload_html_base);
    return html_base;
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let html_base=html_base_make().await;
    HttpServer::new(move ||{
        App::new()
        .app_data(Data::new(Arc::new(html_base.clone())))
        .service(random_img_html)
        .service(target_img_html)
        .service(random)
        .service(
            web::resource("/upload/")
                .route(web::get().to(upload_index))
                .route(web::post().to(upload))
        )
        .service(
            web::resource("/uploadzip/")
                .route(web::get().to(upload_index))
                .route(web::post().to(upload_zip))
        )
    })
    .workers(16)
    .bind(("0.0.0.0",8080))?
    .run()
    .await
}
