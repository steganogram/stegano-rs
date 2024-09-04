# Encodings

## Plain Text Version 2

the raw byte stream starts with 1-Byte `0x01` and will be terminated by 2-Byte `0xffff`
      
## Binary Data Version 2

the raw byte stream starts with 1-Byte `0x02` and will be terminated by

### example

given a string like "Hel" encoded as UTF-8

```txt
String: H           e           l
Hex   : 0x48        0x65        0x6C
Binary: 0b01001000  0b01100101  0b01101100
```