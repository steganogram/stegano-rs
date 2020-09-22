# Changelog

## 🎼 0.4.0

  - ### ✨ Features
  
     - **Add WAV Audio media file support - by [sassman], [pull/6]**
 
       `stegano` has now support for input and output wav audio files (*.wav). This means that hiding secret messages and files are now **not** only possible for png image media files but also wav audio files in the same way. For example like this: 
 
       ```sh
       ❯ stegano hide \
         -i resources/plain/carrier-audio.wav \
         -d resources/secrets/Blah.txt \
            resources/secrets/Blah-2.txt \
         -o secret.wav
       ``` 

       [sassman]: https://github.com/sassman
       [pull/6]: https://github.com/steganogram/stegano-rs/pull/6
       
     - **Add Arch Linux packages - by [orhun], [pull/10]**

        `stegano` can now be installed from available [AUR packages](https://aur.archlinux.org/packages/?O=0&SeB=b&K=stegano&outdated=&SB=n&SO=a&PP=50&do_Search=Go) using an [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers). For example like this:

        ```sh
        ❯ yay -S stegano
        ```
     
       [orhun]: https://github.com/orhun
       [pull/10]: https://github.com/steganogram/stegano-rs/pull/10
  
  - ### 🛠️ Maintenance
  
    - **Update `stegano-core` to latest dependencies - by [sassman], [pull/2]**

       [sassman]: https://github.com/sassman
       [pull/6]: https://github.com/steganogram/stegano-rs/pull/6