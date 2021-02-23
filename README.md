## Getting Started

Download the binary from releases page.

```bash
wget https://github.com/SachinMaharana/polybar-reddit/releases/download/0.1.0/polybar-reddit

chmod +x polybar-reddit

cp polybar-reddit /usr/local/bin

polybar-reddit -h

# Initialize for the first time
# ------------------------------

polybar-reddit --init

# Note the config file location after running the above command(both are required for polybar config):
ls /home/user/.polybarreddit/config

/home/user/.polybarreddit/config/default.toml # edit this to add more subreddits of your choice
/home/user/.polybarreddit/config/current_post.txt
```

## Polybar Config

```bash
[module/reddit]
type = custom/script
exec = /usr/local/bin/polybar-reddit
; interval = 30
content-foreground = ${color.deep-orange}
click-left = < /home/user/.polybarreddit/config/current_post.txt xargs -I % xdg-open %
tail = true
label-maxlen = 100
```

## Roadmap

See the [open issues](https://github.com/sachinmaharana/polybar-reddit/issues) for a list of proposed features (and known issues).

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE` for more information.

<!-- CONTACT -->

## Contact

Project Link: [https://github.com/sachinmaharana/polybar-reddit](https://github.com/sachinmaharana/polybar-reddit)

```

```
