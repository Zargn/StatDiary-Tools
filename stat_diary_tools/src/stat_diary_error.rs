/*
#[derive(Debug)]
pub enum DBAveragesError {
    IoError(io::Error),
    WalkDirError(walkdir::Error),
    InvalidFileName(String),
} */

/*
#[derive(Debug)]
pub enum RegenCachesError {
    InvalidRoot,
    IoError(io::Error),
    FoundUnknownFile(PathBuf),
    FoundUnknownFolder(PathBuf),
}*/

/*
#[derive(Debug)]
pub enum DBError {
    IoError(io::Error),
    CorruptedTagsFile(String),
    UnknownTag(String),
    UnknownId(u16),
    TagAlreadyExists,
    DataBaseBusy,
}*/

/*
TODO: Research good error design

At the moment I am torn between having one large error type for this
entire library or smaller purpose made error types for each part.

I know this library wont be used outside of the statdiary app, but I
still want to practise good design.
It is clear that there are both a lot of shared and a lot of
exclusive errors that can occur in the different functions.

One large error type would make it easier to take care of the
overlapping errors, but it would also mean that all functions that
return said type would appear to possibly return any of the error
types defined there, even those who could never occur in said
function.

Lots of separate error types would instead result in the complete
opposite. It would be easier to understand and handle the errors of
each function, at the cost of more repeated code and more work when
updating code.


*/
