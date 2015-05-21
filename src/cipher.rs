use std::io::Result;
use crypto::md5::Md5;
use crypto::digest::Digest;
use rustc_serialize::base64::FromBase64;


#[derive(Debug)]
pub struct Cipher {
    pub key: String,
}

impl Cipher {
    pub fn new() -> Cipher {
        Cipher { key: String::new() }
    }

    pub fn set_aibang_key(&mut self, key: &str) {
        self.key = format!("aibang{}", key);
    }

    pub fn new_with_aibang_key(key: usize) -> Cipher {
        Cipher { key: format!("aibang{}", key) }
    }

    fn make_translate_table(&self) -> [u8; 256] {
        let mut digest = Md5::new();
        digest.input_str(self.key.as_ref());
        let key_bytes = digest.result_str().into_bytes();

        let mut ret = [0u8; 256];
        for i in 0..256 {
            ret[i] = i as u8;
        }
        let mut k: u16 = 0;
        let mut m: u16 = 0;
        for i in 0..256 {
            k = (key_bytes[m as usize] as u16 + ret[i] as u16 + k) & 255;

            ret.swap(i, k as usize);
            m = (1 + m) % key_bytes.len() as u16
        }
        // println!("key => {:?} \nret=>{:?}", key_bytes, ret.to_vec());
        ret
    }

    fn translate(&self, raw: &[u8], out: &mut [u8]) -> Result<usize> {
        let mut trans_table = self.make_translate_table();

        assert!(out.len() >= raw.len());
        let mut j: u16 = 0;
        let mut k: u16 = 0;
        let out_len = raw.len();
        for i in 0..out_len {
            k = (k + 1) & 255;
            j = (j + trans_table[k as usize] as u16) & 255;

            trans_table.swap(j as usize, k as usize);
            let n: usize = (trans_table[k as usize] as usize + trans_table[j as usize] as usize) & 255;
            out[i] = raw[i] ^ trans_table[n]
        }
        Ok(out_len)
    }

    pub fn decrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let input = input.from_base64().unwrap();
        let mut output = Vec::with_capacity(input.len());
        unsafe { output.set_len(input.len()) }
        try!(self.translate(&input, &mut output));
        Ok(output)
    }

    pub fn decrypt_str(&mut self, input: &str) -> Result<String> {
        self.decrypt(input.as_bytes()).map(|s| String::from_utf8(s).unwrap())
    }

}


#[test]
fn test_cipher_aibang_decrypt() {
    // .decrypt(u'ycCx9MhBlIC3XYEfN4ZZ'))
    let mut cipher = Cipher::new_with_aibang_key(1413772960);
    let out = cipher.decrypt(b"ycCx9MhBlIC3XYEfN4ZZ");

    assert_eq!(cipher.decrypt_str("ycCx9MhBlIC3XYEfN4ZZ").unwrap(), "三家店东口");

}
