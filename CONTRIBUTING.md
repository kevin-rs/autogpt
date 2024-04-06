# 🌍 Contributing.

Contributions are welcome, and they are greatly appreciated! Every little bit helps, and credit will always be given.

## 👶 Getting Started!

Ready to contribute? Here's how to set up `autogpt` for local development.

1. Fork the `autogpt` repo on GitHub.
2. Clone your fork locally:

```sh
git clone https://github.com/kevin-rs/autogpt.git
```

3. Navigate to the recently created directory:

```sh
cd autogpt
```

4. Build the container:

```sh
docker build -t autogpt .
```

5. Run container:

```sh
docker run -i -e GEMINI_API_KEY=<your_gemini_api_key> -t autogpt:latest
```

6. Create a branch for local development:

```sh
git checkout -b name-of-your-bugfix-or-feature
```

Now you can make your changes locally.

7. Commit your changes and push your branch to GitHub:

```sh
git add .
git commit -m "Your detailed description of your changes."
git push origin name-of-your-bugfix-or-feature
```

8. Submit a pull request through the GitHub website.

## 📙 Pull Request Guidelines.

Before you submit a pull request, check that it meets these guidelines:

1. The pull request should include tests, if applicable.

Thank you for helping us improve!