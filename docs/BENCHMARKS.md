## commit 5f1a7f20765a7abbb64e18c2b7eedfbf30eb418d
```sh
Running target/release/deps/decoder_benchmark-9e4572eff811da8d
SteganoCore Image Decoding
                        time:   [629.97 ns 636.77 ns 645.08 ns]
                        change: [-14.878% -13.636% -12.362%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

SteganoCore Audio Decoding
                        time:   [2.5929 us 2.5952 us 2.5977 us]
                        change: [-72.051% -71.865% -71.692%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe

     Running target/release/deps/encoder_benchmark-9ec9f54bb533cdc3
SteganoCore Image Encoding to memory
                        time:   [607.06 us 612.78 us 619.02 us]
                        change: [-11.419% -8.6893% -5.9210%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  10 (10.00%) high mild
  4 (4.00%) high severe

SteganoCore Audio Encoding to memory
                        time:   [214.06 ns 214.62 ns 215.25 ns]
                        change: [+143.57% +145.20% +146.88%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
```

## commit 56e23162c08de48d773c292ed136c259b4e898a0

```sh
Running target/release/deps/decoder_benchmark-5cbabbf84906527c
Gnuplot not found, using plotters backend
SteganoCore Image Decoding
                        time:   [563.50 ns 565.75 ns 568.55 ns]
                        change: [-6.0991% -4.4367% -2.8252%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

SteganoCore Audio Decoding
                        time:   [10.813 us 10.860 us 10.914 us]
                        change: [-11.084% -8.9057% -6.8391%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

SteganoCore Audio Encoding
                        time:   [174.65 us 182.18 us 190.23 us]
                        change: [+13.802% +26.589% +42.095%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
```

## commit ea6c2d9a395e13b20c3aaf96d64537a643246829:
```sh
Running target/release/deps/decoder_benchmark-5cbabbf84906527c
Gnuplot not found, using plotters backend
SteganoCore::LSBCodec for resources/with_text/hello_world.png (decode)
                        time:   [635.24 ns 639.12 ns 644.20 ns]
                        change: [+1.2547% +2.4118% +3.5136%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  6 (6.00%) high severe

stegano_core::audio::LSBCodec decoder
                        time:   [12.112 us 12.622 us 13.269 us]
                        change: [+7.2557% +10.335% +14.802%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

stegano_core::audio::LSBCodec encoding
                        time:   [99.087 us 105.48 us 114.03 us]
                        change: [-46.688% -42.952% -37.539%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
```

## commit 7fb64053addd00a0897713416667d9312560aaa6
```sh
Running target/release/deps/decoder_benchmark-5cbabbf84906527c
Gnuplot not found, using plotters backend
SteganoCore::LSBCodec for resources/with_text/hello_world.png (decode)
                        time:   [621.97 ns 627.36 ns 633.02 ns]
                        change: [-2.6063% -0.8664% +1.4211%] (p = 0.36 > 0.05)
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe

stegano_core::audio::LSBCodec decoder
                        time:   [11.691 us 11.747 us 11.806 us]
                        change: [-2.7207% +1.4449% +5.3438%] (p = 0.50 > 0.05)
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) high mild
  10 (10.00%) high severe

stegano_core::audio::LSBCodec encoding
                        time:   [122.43 us 129.44 us 138.00 us]
                        change: [+6.9255% +16.046% +24.869%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

