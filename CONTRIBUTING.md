## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any
contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also open
an issue with the tag "enhancement".
Remember to give the project a star! Thanks again!

### Development workflow

Before submitting a pull request, make sure your changes pass the linter and the test suite:

```bash
cargo clippy
cargo test
```

To run the web app locally while developing:

```bash
trunk serve --port 8080
```

Then open <http://localhost:8080/index.html#dev> in your browser.

### Pull request checklist

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
