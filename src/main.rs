//#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use dotenvy::dotenv;
use std::env;
use tera::Tera;
use diesel::prelude::*;
use diesel::pg::PgConnection;

//use self::models::{Post, NewPost, NewPostHandler};
use self::models::{Post, NewPostHandler};
//use self::schema::posts;
use self::schema::posts::dsl::*;

use diesel::r2d2::{self, ConnectionManager, Pool};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/")]
async fn index(template_manager: web::Data<tera::Tera>, pool: web::Data<DbPool>) -> impl Responder{
    let mut conn = pool.get().expect("Problemas al traer la base de datos");
    
    match web::block(move || {posts.load::<Post>(&mut conn)}).await{
        Ok(data) => {
            let data = data.unwrap();
            let mut context = tera::Context::new();
            context.insert("variable", "Así se insertan mensajes");
            context.insert("posts", &data);
            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("index.html", &context).unwrap()
            )
            //return HttpResponse::Ok().body(format!("{:?}", data));
        },
        Err(err) => HttpResponse::Ok().body(format!("Error al recibir la data: {}", err))
    }
}
#[get("/post/{blog_slug}")]
async fn get_post(
    template_manager: web::Data<tera::Tera>, 
    pool: web::Data<DbPool>, 
    blog_slug: web::Path<String>) -> impl Responder{

    let mut conn = pool.get().expect("Problemas al traer la base de datos");
    let url_slug = blog_slug.into_inner();
    
    match web::block(move || {posts.filter(slug.eq(url_slug)).load::<Post>(&mut conn)}).await{
        Ok(data) => {
            let data = data.unwrap();
            if data.len() == 0{
                return HttpResponse::NotFound().finish();
            }
            let data = &data[0];
            let mut context = tera::Context::new();
            context.insert("variable", "Así se insertan mensajes");
            context.insert("post", data);
            HttpResponse::Ok().content_type("text/html").body(
                template_manager.render("posts.html", &context).unwrap()
            )
        },
        Err(err) => HttpResponse::Ok().body(format!("Error al recibir la data: {}", err))
    }
}

#[post("/new_post")]
async fn new_post(pool: web::Data<DbPool>, item: web::Json<NewPostHandler>) -> impl Responder{
    let conn= pool.get().expect("Problemas al traer la base de datos");

    println!("{:?}", item);
    match web::block(move || {Post::create_post(conn, &item)}).await{
        Ok(data) => {
            HttpResponse::Ok().body(format!("{:?}", data))
        },
        Err(err) => HttpResponse::Ok().body(format!("Error al recibir la data: {}", err))
    }
}
//docker run -d --name blog-rust -e "PORT=" -e "DEBUG=0" -p 8007:8765 web:latest
//docker run -d --name blog-rust -e "PORT=8080" -e "DEBUG=0" -p 8080:8080 web:latest
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url=env::var("DATABASE_URL").expect("db url no encontrada");
    let port=env::var("PORT").expect("Puerto no encontrada");
    let port: u16 = port.parse().unwrap();

    let connection = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder().build(connection).expect("No se pudo construir la pool");

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
        .service(index)
        .service(new_post)
        .service(get_post)
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(tera))
    }).bind(("0.0.0.0", port))?.run().await
}
