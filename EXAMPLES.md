# 📌 Examples

This repository contains a list of notebooks examples on how to use the sdk and or the cli. To use the notebooks in this repository, you need to set up your environment. Follow these steps to get started:

1. Clone the repository to your local machine:

   ```sh
   git clone https://github.com/kevin-rs/autogpt.git
   ```

1. Install the required dependencies and libraries. Make sure you have [`Rust`](https://rustup.rs/), [`Jupyter Notebook`](https://jupyter.org/install), and [`evcxr_jupyter`](https://github.com/evcxr/evcxr/blob/main/evcxr_jupyter/README.md) installed on your system.

   ```sh
   # Install a Rust toolchain (e.g. nightly):
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
 
   # Install Jupyter Notebook
   pip install notebook
 
   # Install evcxr_jupyter
   cargo install evcxr_jupyter
   evcxr_jupyter --install 
   ```

1. Navigate to the cloned repository and build the project:

   ```sh
   cd gems
   cargo build --release --all-features
   ```

1. Start Jupyter Notebook:

   ```sh
   jupyter notebook
   ```

1. Access the notebooks in your web browser by clicking on the notebook file you want to explore.

| ID | Example | Open on GitHub | Launch on Binder | Launch on Colab |
|----|---------------|-----------|:-------------|-------------|
| 1  | **Basic** | [![Github](https://img.shields.io/badge/launch-Github-181717.svg?logo=github&logoColor=white)](./examples/basic.ipynb) | [![Binder](https://mybinder.org/badge_logo.svg)](https://mybinder.org/v2/gh/kevin-rs/autogpt/main?filepath=examples/basic.ipynb) |  [![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/kevin-rs/autogpt/blob/main/examples/basic.ipynb) |

---
