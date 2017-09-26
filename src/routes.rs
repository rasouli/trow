use std::hash::{Hash, Hasher};
use std::str;
use std::io::Read;
use std::fs::File;
use std::string::ToString;
use rocket;
use uuid::Uuid;
use ring::{digest, test};

use errors;
use response::{Responses, MaybeResponse};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_manifest,
        check_image_manifest,
        get_blob,
        post_blob_uuid,
        check_existing_layer,
        get_upload_progress,
        put_blob,
        patch_blob,
        delete_upload,
        post_blob_upload,
        delete_blob,
        put_image_manifest,
        get_catalog,
        get_image_tags,
        delete_image_manifest,
    ]
}

pub fn errors() -> Vec<rocket::Catcher> {
    errors![
        err_400,
        err_404,
        ]
        
}

#[error(400)]
fn err_400() -> MaybeResponse<Responses> {
    let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    MaybeResponse::err(errors)
}

#[error(404)]
fn err_404() -> MaybeResponse<Responses> {
    let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    MaybeResponse::err(errors)
}

/**
Routes of a 2.0 Registry

Version Check of the registry
GET /v2/

# Responses
200 - We Exist (and you are authenticated)
401 - Please Authorize (WWW-Authenticate header with instuctions).

# Headers
Docker-Distribution-API-Version: registry/2.0
*/

/// Some docs for this function
#[get("/v2")]
fn get_v2root() -> MaybeResponse<Responses> {
    MaybeResponse::ok(Responses::Empty)
}

/*
---
Pulling an image
GET /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Client Headers
Accept: manifest-version

# Headers
Accept: manifest-version
?Docker-Content-Digest: digest of manifest file

# Returns
200 - return the manifest
404 - manifest not known to the registry
*/
#[get("/v2/<name>/<repo>/manifests/<reference>")]
fn get_manifest(
    name: String,
    repo: String,
    reference: String,
) -> MaybeResponse<Responses> {
    info!("Getting Manifest");
    let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    match reference.as_str() {
        "good" => MaybeResponse::ok(Responses::Empty),
        _ => MaybeResponse::err(errors),
    }
}
/*

---
Check for existance
HEAD /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Headers
Content-Length: size of manifest
?Docker-Content-Digest: digest of manifest file

# Returns
200 - manifest exists
404 - manifest does not exist
 */
#[head("/v2/<name>/<repo>/manifests/<reference>")]
fn check_image_manifest(name: String, repo: String, reference: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}

/*
---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downoaded

# Responses
200 - blob is downloaded
307 - redirect to another service for downloading[1]
 */
#[get("/v2/<name>/<repo>/blobs/<digest>")]
fn get_blob(name: String, repo: String, digest: String) -> MaybeResponse<Responses> {
    info!("Getting Blob");
    let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    match digest.as_str() {
        "good" => MaybeResponse::ok(Responses::Empty),
        _ => MaybeResponse::err(errors),
    }
}

/**

---
Pushing a Layer
POST /v2/<name>/blobs/uploads/
name - name of repository

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Content-Length: 0
Docker-Upload-UUID: <uuid>

# Returns
202 - accepted
*/
#[post("/v2/<name>/<repo>/blobs/uploads/<uuid>")]
fn post_blob_uuid(name: String, repo: String, uuid: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}

/*
---
Check for existing layer
HEAD /v2/<name>/blobs/<digest>
name - name of repository
digest - digest of blob to be checked

# Headers
Content-Length: <length of blob>
Docker-Content-Digest: <digest>

# Returns
200 - exists
404 - does not exist
 */
#[head("/v2/<name>/<repo>/blobs/<digest>")]
fn check_existing_layer(name: String, repo: String, digest: String) ->
    MaybeResponse<Responses> {
        debug!("Checking if {}/{} exists...", name, repo);
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}

/*
---
Upload Progress
GET /v2/<name>/blobs/uploads/<uuid>
name - name of registry
uuid - unique id for the upload that is to be checked

# Client Headers
Host: <registry host>

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Docker-Upload-UUID: <uuid>

# Returns
204
 */
#[get("/v2/<name>/<repo>/blobs/uploads/<uuid>")]
fn get_upload_progress(name: String, repo: String, uuid: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*

---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>
---
Chunked Upload (Don't implement until Monolithic works)
Must be implemented as docker only supports this
PATCH /v2/<name>/blobs/uploads/<uuid>
Content-Length: <size of chunk>
Content-Range: <start of range>-<end of range>
Content-Type: application/octet-stream

<Layer Chunk Binary Data>
 */

#[derive_FromForm]
#[derive(Debug)]
struct DigestStruct {
    query: bool,
    digest: String,
}

// TODO change this to return a type-safe thing rather than just 'String'
fn scratch_path(uuid: &String) -> String {
    format!("data/scratch/{}", uuid)
}

// TODO change this to return a type-safe thing rather than just 'String'
fn hash_file(absolute_directory: String) -> Result<String, String> {
    debug!("Hashing file: {}", absolute_directory);
    match File::open(&absolute_directory) {
        Ok(mut file) => {
            let mut vec_file = &mut Vec::new();
            let _ = file.read_to_end(&mut vec_file);
            let sha = digest::digest(&digest::SHA256, &vec_file);

            // HACK: needs a fix of some description
            Ok(format!("{:?}", sha).to_lowercase())
        }
        Err(_) => Err(format!("could not open file: {}", absolute_directory))
    }
}

#[put("/v2/<name>/<repo>/blobs/uploads/<uuid>?<digest>")] // capture digest query string
fn put_blob(name: String, repo: String, uuid: String, digest: DigestStruct) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        debug!("Completing layer upload with digest: {}", digest.digest);
        let hash = match hash_file(scratch_path(&uuid)) {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };
        debug!("File Hash: {}", hash);

        match assert_eq!(hash, digest.digest) {
            () => MaybeResponse::err(errors)
        }


        // hash uuid from scratch, if success, copy over to layers
        // UuidAccept
        // match digest.digest.eq(hash) {
        //     True => MaybeResponse::err(errors),
        //     False => True => MaybeResponse::err(errors).
        // }
}

#[patch("/v2/<name>/<repo>/blobs/uploads/<uuid>", data="<chunk>")]
fn patch_blob(name: String, repo: String, uuid: String, chunk: rocket::data::Data) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        let absolute_file = scratch_path(&uuid);
        debug!("Streaming out to {}", absolute_file);
        let file = chunk.stream_to_file(absolute_file);

        match file {
            Ok(_) => {
                let right = match file.map(|x| x.to_string()) {
                    Ok(x) => x.parse::<u32>().unwrap(),
                    Err(_) => 0,
                };
                MaybeResponse::ok(Responses::Uuid {uuid, name, repo, left: 0, right })
            },
            Err(_) => MaybeResponse::err(errors)
        }
}

/*


---
Cancelling an upload
DELETE /v2/<name>/blobs/uploads/<uuid>

 */

#[delete("/v2/<name>/<repo>/blobs/uploads/<uuid>")]
fn delete_upload(name: String, repo: String, uuid: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*
---
Cross repo blob mounting (validate how regularly this is used)
POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<repository name>

 */

#[post("/v2/<name>/<repo>/blobs/uploads")]
fn post_blob_upload(name: String, repo: String) ->
    MaybeResponse<Responses> {
        let uuid = Uuid::new_v4();
        info!("Using Uuid: {:?}", uuid);
        MaybeResponse::ok(Responses::Uuid {
            uuid: uuid.to_string(),
            name,
            repo,
            left: 0,
            right: 0,
        })
}
/*

---
Delete a layer
DELETE /v2/<name>/blobs/<digest>

*/
#[delete("/v2/<name>/<repo>/blobs/<digest>")]
fn delete_blob(name: String, repo: String, digest: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

*/
#[put("/v2/<name>/<repo>/manifests/<reference>")]
fn put_image_manifest(name: String, repo: String, reference: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*
---
Listing Repositories
GET /v2/_catalog

*/
#[get("/v2/_catalog")]
fn get_catalog() ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*
---
Listing Image Tags
GET /v2/<name>/tags/list

*/
#[delete("/v2/<name>/<repo>/tags/list")]
fn get_image_tags(name: String, repo: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}
/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>

*/
#[delete("/v2/<name>/<repo>/manifests/<reference>")]
fn delete_image_manifest(name: String, repo: String, reference: String) ->
    MaybeResponse<Responses> {
        let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(errors)
}

/*
---
[1]: Could possibly be used to redirect a client to a local cache
*/