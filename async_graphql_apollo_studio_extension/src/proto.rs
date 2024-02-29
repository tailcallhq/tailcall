#![allow(rustdoc::all)]
#![allow(clippy::all)]

pub mod report {
    tonic::include_proto!("report");
}

pub mod agent {
    tonic::include_proto!("agent");
}
