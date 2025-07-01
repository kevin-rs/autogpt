fn main() {
    prost_build::compile_protos(&["proto/iac.proto"], &["proto"])
        .expect("Failed to compile protobuf");
}
