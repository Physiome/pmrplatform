use gix::{
    Commit,
    Object,
    ObjectDetached,
    actor::SignatureRef,
    object::Kind,
    objs::{
        BlobRef,
        CommitRef,
        TreeRef,
        WriteTo as _,
    },
};

use crate::repo::*;
use super::{
    IdCommitRef,
    PathObject,
    PathObjectDetached,
};
use super::util::*;

impl<'a> PathObject<'a> {
    pub fn new(path: String, object: Object<'a>) -> Self {
        Self { path, object }
    }
}

impl PathObjectDetached {
    pub fn new(path: String, object: ObjectDetached) -> Self {
        Self { path, object }
    }
}

impl<'a> TryFrom<&'a Commit<'a>> for IdCommitRef<'a> {
    type Error = gix::objs::decode::Error;

    fn try_from(item: &'a Commit<'a>) -> Result<Self, gix::objs::decode::Error> {
        let commit_id = item.id.to_string();
        let commit = item.decode()?;
        Ok(IdCommitRef { commit_id, commit })
    }
}

impl From<Object<'_>> for ObjectInfo {
    fn from(object: Object<'_>) -> Self {
        object_to_info(&object)
    }
}

impl From<ObjectDetached> for ObjectInfo {
    fn from(object: ObjectDetached) -> Self {
        object_detached_to_info(&object)
    }
}

impl From<PathObject<'_>> for ObjectInfo {
    fn from(item: PathObject<'_>) -> Self {
        pathobject_to_info(&item)
    }
}

impl From<PathObject<'_>> for PathObjectInfo {
    fn from(item: PathObject<'_>) -> Self {
        pathobject_to_pathobjectinfo(&item)
    }
}

impl From<&PathObject<'_>> for PathObjectInfo {
    fn from(item: &PathObject<'_>) -> Self {
        pathobject_to_pathobjectinfo(&item)
    }
}

impl From<PathObjectDetached> for ObjectInfo {
    fn from(item: PathObjectDetached) -> Self {
        pathobjectdetached_to_info(&item)
    }
}

impl From<PathObjectDetached> for PathObjectInfo {
    fn from(item: PathObjectDetached) -> Self {
        pathobjectdetached_to_pathobjectinfo(&item)
    }
}

impl From<&PathObjectDetached> for PathObjectInfo {
    fn from(item: &PathObjectDetached) -> Self {
        pathobjectdetached_to_pathobjectinfo(&item)
    }
}

impl TryFrom<Commit<'_>> for CommitInfo {
    type Error = gix::objs::decode::Error;

    fn try_from(item: Commit<'_>) -> Result<Self, Self::Error> {
        commit_to_commitinfo(&item)
    }
}

impl TryFrom<&Commit<'_>> for CommitInfo {
    type Error = gix::objs::decode::Error;

    fn try_from(item: &Commit<'_>) -> Result<Self, Self::Error> {
        commit_to_commitinfo(&item)
    }
}

impl TryFrom<ObjectDetached> for CommitInfo {
    type Error = gix::objs::decode::Error;

    fn try_from(item: ObjectDetached) -> Result<Self, Self::Error> {
        objectdetached_to_commitinfo(&item)
    }
}

impl TryFrom<&ObjectDetached> for CommitInfo {
    type Error = gix::objs::decode::Error;

    fn try_from(item: &ObjectDetached) -> Result<Self, Self::Error> {
        objectdetached_to_commitinfo(&item)
    }
}


// These assume the blobs are all contained because the conversion to
// the Ref equivalent currently drops information for gix, and to make
// the internal usage consistent, the raw object is passed.
fn obj_blob_to_fileinfo(git_object: &Object, path: Option<&str>) -> FileInfo {
    let blob = BlobRef::from_bytes(&git_object.data)
        .expect("should have been verified as a blob");
    FileInfo {
        size: blob.size() as u64,
        binary: is_binary(blob.data),
        mime_type: path
            .and_then(|path| mime_guess::from_path(path).first_raw())
            .unwrap_or("application/octet-stream")
            .to_string(),
    }
}

fn objd_blob_to_fileinfo(git_object: &ObjectDetached, path: Option<&str>) -> FileInfo {
    let blob = BlobRef::from_bytes(&git_object.data)
        .expect("should have been verified as a blob");
    FileInfo {
        size: blob.size() as u64,
        binary: is_binary(blob.data),
        mime_type: path
            .and_then(|path| mime_guess::from_path(path).first_raw())
            .unwrap_or("application/octet-stream")
            .to_string(),
    }
}

fn obj_tree_to_treeinfo(git_object: &Object) -> TreeInfo {
    let tree = TreeRef::from_bytes(&git_object.data)
        .expect("should have been verfieid as a well-formed tree");
    TreeInfo {
        filecount: tree.entries.len() as u64,
        entries: tree.entries.iter().map(|entry| TreeEntryInfo {
            filemode: std::str::from_utf8(
                entry.mode
                    .kind()
                    .as_octal_str()
            )
                .expect("these are standard ascii numbers")
                .to_string(),
            kind: entry.mode.as_str().to_string(),
            id: entry.oid.to_string(),
            name: entry.filename.to_string(),
        }).collect(),
    }
}

fn objd_tree_to_treeinfo(git_object: &ObjectDetached) -> TreeInfo {
    let tree = TreeRef::from_bytes(&git_object.data)
        .expect("should have been verfieid as a well-formed tree");
    TreeInfo {
        filecount: tree.entries.len() as u64,
        entries: tree.entries.iter().map(|entry| TreeEntryInfo {
            filemode: std::str::from_utf8(
                entry.mode
                    .kind()
                    .as_octal_str()
            )
                .expect("these are standard ascii numbers")
                .to_string(),
            kind: entry.mode.as_str().to_string(),
            id: entry.oid.to_string(),
            name: entry.filename.to_string(),
        }).collect(),
    }
}

fn obj_commit_to_commitinfo(git_object: &Object) -> CommitInfo {
    commitref_id_to_commitinfo(
        git_object.id.to_string(),
        CommitRef::from_bytes(&git_object.data)
            .expect("should have been verified as a well-formed commit"),
    )
}

fn objd_commit_to_commitinfo(git_object: &ObjectDetached) -> CommitInfo {
    commitref_id_to_commitinfo(
        git_object.id.to_string(),
        CommitRef::from_bytes(&git_object.data)
            .expect("should have been verified as a well-formed commit"),
    )
}

fn format_signature_ref(
    value: &SignatureRef,
) -> String {
    format!("{} <{}>", value.name, value.email)
}

fn commitref_id_to_commitinfo(
    commit_id: String,
    commit: CommitRef,
) -> CommitInfo {
    CommitInfo {
        commit_id: commit_id,
        author: format_signature_ref(&commit.author()),
        committer: format_signature_ref(&commit.committer()),
    }
}

fn commit_to_commitinfo(
    commit: &Commit<'_>,
) -> Result<CommitInfo, gix::objs::decode::Error> {
    Ok(CommitInfo {
        commit_id: commit.id().to_string(),
        author: format_signature_ref(&commit.author()?),
        committer: format_signature_ref(&commit.committer()?),
    })
}

fn objectdetached_to_commitinfo(
    object: &ObjectDetached,
) -> Result<CommitInfo, gix::objs::decode::Error> {
    let commit = CommitRef::from_bytes(&object.data)?;
    Ok(CommitInfo {
        commit_id: object.id.to_string(),
        author: format_signature_ref(&commit.author),
        committer: format_signature_ref(&commit.committer),
    })
}

fn object_to_info(git_object: &Object) -> ObjectInfo {
    match git_object.kind {
        Kind::Blob => ObjectInfo::FileInfo(
            obj_blob_to_fileinfo(&git_object, None),
        ),
        Kind::Tree => ObjectInfo::TreeInfo(
            obj_tree_to_treeinfo(&git_object),
        ),
        Kind::Commit => ObjectInfo::CommitInfo(
            obj_commit_to_commitinfo(&git_object),
        ),
        Kind::Tag => ObjectInfo::Unknown,
    }
}

fn object_detached_to_info(git_object: &ObjectDetached) -> ObjectInfo {
    match git_object.kind {
        Kind::Blob => ObjectInfo::FileInfo(
            objd_blob_to_fileinfo(&git_object, None),
        ),
        Kind::Tree => ObjectInfo::TreeInfo(
            objd_tree_to_treeinfo(&git_object),
        ),
        Kind::Commit => ObjectInfo::CommitInfo(
            objd_commit_to_commitinfo(&git_object),
        ),
        Kind::Tag => ObjectInfo::Unknown,
    }
}

fn pathobject_to_info(item: &PathObject) -> ObjectInfo {
    match item.object.kind {
        Kind::Blob => ObjectInfo::FileInfo(
            obj_blob_to_fileinfo(&item.object, Some(&item.path)),
        ),
        Kind::Tree => ObjectInfo::TreeInfo(
            obj_tree_to_treeinfo(&item.object),
        ),
        _ => ObjectInfo::Unknown,
    }
}

fn pathobjectdetached_to_info(item: &PathObjectDetached) -> ObjectInfo {
    match item.object.kind {
        Kind::Blob => ObjectInfo::FileInfo(
            objd_blob_to_fileinfo(&item.object, Some(&item.path)),
        ),
        Kind::Tree => ObjectInfo::TreeInfo(
            objd_tree_to_treeinfo(&item.object),
        ),
        _ => ObjectInfo::Unknown,
    }
}

fn pathobject_to_pathobjectinfo(item: &PathObject) -> PathObjectInfo {
    match item.object.kind {
        Kind::Blob => PathObjectInfo::FileInfo(
            obj_blob_to_fileinfo(&item.object, Some(&item.path)),
        ),
        Kind::Tree => PathObjectInfo::TreeInfo(
            obj_tree_to_treeinfo(&item.object),
        ),
        _ => PathObjectInfo::Unknown,
    }
}

fn pathobjectdetached_to_pathobjectinfo(item: &PathObjectDetached) -> PathObjectInfo {
    match item.object.kind {
        Kind::Blob => PathObjectInfo::FileInfo(
            objd_blob_to_fileinfo(&item.object, Some(&item.path)),
        ),
        Kind::Tree => PathObjectInfo::TreeInfo(
            objd_tree_to_treeinfo(&item.object),
        ),
        _ => PathObjectInfo::Unknown,
    }
}
