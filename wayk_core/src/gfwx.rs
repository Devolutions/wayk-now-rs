use gfwx::Encoder;
use gfwx::Filter;
use gfwx::Header;
use gfwx::Intent;
use gfwx::Quantization;
use num_derive::FromPrimitive;
use wayk_proto_derive::{Decode, Encode};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct NowGfwxParams {
    pub version: u32,
    pub layers: u16,
    pub channels: u16,
    pub bit_depth: u8,
    pub is_signed: bool,
    pub intent: Intent,
    pub metadata_size: u32,
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum NowGWFXFilter {
    Linear = 0,
    Cubic = 1,
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum NowGWFXQuantization {
    Scalar = 0,
}

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum NowGWFXEncoder {
    Turbo = 0,
    Fast = 1,
    Contextual = 2,
}

#[repr(C)]
#[derive(Encode, Decode, Clone, Debug)]
pub struct NowGfwxHeader {
    pub magic: u32,
    pub version: u8,
    pub offset: u8,
    pub flags: u16,

    pub quality_level: u16,
    pub chroma_scale: u8,
    pub block_size: u8,
    pub filter: NowGWFXFilter,
    pub quantization: NowGWFXQuantization,
    pub encoder: NowGWFXEncoder,
    pub boost: u8,

    pub image_width: u32,
    pub image_height: u32,
    pub layer_count: u16,
    pub channel_count: u16,

    pub transform: u32,
    pub color_flags: u32,
}

impl From<NowGWFXFilter> for Filter {
    fn from(now_gfwx_filter: NowGWFXFilter) -> Filter {
        match now_gfwx_filter {
            NowGWFXFilter::Linear => Filter::Linear,
            NowGWFXFilter::Cubic => Filter::Cubic,
        }
    }
}

impl From<NowGWFXQuantization> for Quantization {
    fn from(now_gfwx_quantization: NowGWFXQuantization) -> Quantization {
        match now_gfwx_quantization {
            NowGWFXQuantization::Scalar => Quantization::Scalar,
        }
    }
}

impl From<NowGWFXEncoder> for Encoder {
    fn from(now_gfwx_encoder: NowGWFXEncoder) -> Encoder {
        match now_gfwx_encoder {
            NowGWFXEncoder::Turbo => Encoder::Turbo,
            NowGWFXEncoder::Fast => Encoder::Fast,
            NowGWFXEncoder::Contextual => Encoder::Contextual,
        }
    }
}

impl NowGfwxHeader {
    pub fn to_gfwx_header_with_params(&self, params: NowGfwxParams) -> Header {
        Header {
            version: params.version,
            width: self.image_width,
            height: self.image_height,
            layers: params.layers,
            channels: params.channels,
            bit_depth: params.bit_depth,
            is_signed: params.is_signed,
            quality: self.quality_level,
            chroma_scale: self.chroma_scale,
            block_size: self.block_size,
            filter: Filter::from(self.filter),
            quantization: Quantization::from(self.quantization),
            encoder: Encoder::from(self.encoder),
            intent: params.intent,
            metadata_size: params.metadata_size,
            channel_size: self.image_width as usize * self.image_height as usize,
            image_size: self.image_width as usize
                * self.image_height as usize
                * params.layers as usize
                * params.channels as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayk_proto::serialization::Decode;

    const ENCODED_NOW_GFWX_HEADER: [u8; 36] = [
        0x47, 0x46, 0x57, 0x58, 0x02, 0x24, 0x00, 0x00, 0x40, 0x00, 0x04, 0x07, 0x00, 0x00, 0x01, 0x08, 0x24, 0x01,
        0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03, 0x00, 0x41, 0x37, 0x31, 0x31, 0x00, 0x00, 0x00, 0x00,
    ];

    const DECODED_NOW_GFWX_HEADER: NowGfwxHeader = NowGfwxHeader {
        magic: 1482114631,
        version: 2,
        offset: 36,
        flags: 0,
        quality_level: 64,
        chroma_scale: 4,
        block_size: 7,
        filter: NowGWFXFilter::Linear,
        quantization: NowGWFXQuantization::Scalar,
        encoder: NowGWFXEncoder::Fast,
        boost: 8,
        image_width: 292,
        image_height: 40,
        layer_count: 1,
        channel_count: 3,
        transform: 825308993,
        color_flags: 0,
    };

    const NOW_GFWX_PARAMS: NowGfwxParams = NowGfwxParams {
        version: 1,
        layers: 1,
        channels: 3,
        bit_depth: 8,
        is_signed: false,
        intent: Intent::RGB,
        metadata_size: 0,
    };

    const HEADER: Header = Header {
        version: NOW_GFWX_PARAMS.version,
        width: DECODED_NOW_GFWX_HEADER.image_width,
        height: DECODED_NOW_GFWX_HEADER.image_height,
        layers: NOW_GFWX_PARAMS.layers,
        channels: NOW_GFWX_PARAMS.channels,
        bit_depth: NOW_GFWX_PARAMS.bit_depth,
        is_signed: NOW_GFWX_PARAMS.is_signed,
        quality: DECODED_NOW_GFWX_HEADER.quality_level,
        chroma_scale: DECODED_NOW_GFWX_HEADER.chroma_scale,
        block_size: DECODED_NOW_GFWX_HEADER.block_size,
        filter: Filter::Linear,
        quantization: Quantization::Scalar,
        encoder: Encoder::Fast,
        intent: NOW_GFWX_PARAMS.intent,
        metadata_size: NOW_GFWX_PARAMS.metadata_size,
        channel_size: DECODED_NOW_GFWX_HEADER.image_width as usize * DECODED_NOW_GFWX_HEADER.image_height as usize,
        image_size: DECODED_NOW_GFWX_HEADER.image_width as usize
            * DECODED_NOW_GFWX_HEADER.image_height as usize
            * NOW_GFWX_PARAMS.layers as usize
            * NOW_GFWX_PARAMS.channels as usize,
    };

    #[test]
    fn now_gfwx_header_decode() {
        let now_gfwx_header = NowGfwxHeader::decode(&ENCODED_NOW_GFWX_HEADER).unwrap();

        assert_eq!(DECODED_NOW_GFWX_HEADER.magic, now_gfwx_header.magic);
        assert_eq!(DECODED_NOW_GFWX_HEADER.version, now_gfwx_header.version);
        assert_eq!(DECODED_NOW_GFWX_HEADER.offset, now_gfwx_header.offset);
        assert_eq!(DECODED_NOW_GFWX_HEADER.flags, now_gfwx_header.flags);
        assert_eq!(DECODED_NOW_GFWX_HEADER.quality_level, now_gfwx_header.quality_level);
        assert_eq!(DECODED_NOW_GFWX_HEADER.chroma_scale, now_gfwx_header.chroma_scale);
        assert_eq!(DECODED_NOW_GFWX_HEADER.block_size, now_gfwx_header.block_size);
        assert_eq!(DECODED_NOW_GFWX_HEADER.filter, now_gfwx_header.filter);
        assert_eq!(DECODED_NOW_GFWX_HEADER.quantization, now_gfwx_header.quantization);
        assert_eq!(DECODED_NOW_GFWX_HEADER.encoder, now_gfwx_header.encoder);
        assert_eq!(DECODED_NOW_GFWX_HEADER.boost, now_gfwx_header.boost);
        assert_eq!(DECODED_NOW_GFWX_HEADER.image_width, now_gfwx_header.image_width);
        assert_eq!(DECODED_NOW_GFWX_HEADER.image_height, now_gfwx_header.image_height);
        assert_eq!(DECODED_NOW_GFWX_HEADER.layer_count, now_gfwx_header.layer_count);
        assert_eq!(DECODED_NOW_GFWX_HEADER.channel_count, now_gfwx_header.channel_count);
        assert_eq!(DECODED_NOW_GFWX_HEADER.transform, now_gfwx_header.transform);
        assert_eq!(DECODED_NOW_GFWX_HEADER.color_flags, now_gfwx_header.color_flags);
    }

    #[test]
    fn header_decode() {
        let header = DECODED_NOW_GFWX_HEADER.to_gfwx_header_with_params(NOW_GFWX_PARAMS);

        assert_eq!(header.version, HEADER.version);
        assert_eq!(header.width, HEADER.width);
        assert_eq!(header.height, HEADER.height);
        assert_eq!(header.layers, HEADER.layers);
        assert_eq!(header.channels, HEADER.channels);
        assert_eq!(header.bit_depth, HEADER.bit_depth);
        assert_eq!(header.is_signed, HEADER.is_signed);
        assert_eq!(header.quality, HEADER.quality);
        assert_eq!(header.chroma_scale, HEADER.chroma_scale);
        assert_eq!(header.block_size, HEADER.block_size);
        assert_eq!(header.filter, HEADER.filter);
        assert_eq!(header.quantization, HEADER.quantization);
        assert_eq!(header.encoder, HEADER.encoder);
        assert_eq!(header.intent, HEADER.intent);
        assert_eq!(header.metadata_size, HEADER.metadata_size);
        assert_eq!(header.channel_size, HEADER.channel_size);
        assert_eq!(header.image_size, HEADER.image_size);
    }
}
