pub mod buf {
    pub mod validate {
        #![allow(clippy::len_without_is_empty)]
        tonic::include_proto!("buf.validate");
    }
}

pub mod google {
    pub mod api {
        tonic::include_proto!("google.api");
    }
}

pub mod tea {
    pub mod v1 {
        #![allow(clippy::large_enum_variant)]
        tonic::include_proto!("tea.v1");
    }
}
