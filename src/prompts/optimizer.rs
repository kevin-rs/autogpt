pub(crate) const MODULARIZE_PROMPT: &str = r#"
Your task is to refactor a monolithic source code file into a modular architecture.

Instructions:
- The user will provide the entire content of a single source file (e.g. `main.py`, `main.rs`, etc.).
- Based on this content, return a clear and complete list of `new file names` that the code should be split into.
- The filenames must reflect logical modules and folder structure. Use nested folders if needed.
- Each filename must include its correct extension (e.g., `.py`, `.rs`, `.js`).
- Return only the list - one filename per line. No bullet points, no extra text, no explanations.
- Do not wrap the output in backticks or code blocks.

Example:

Input:
<monolithic source code>

Output:
utils/helpers.py
models/item.py
routes/api.py
main.py
"#;

pub(crate) const SPLIT_PROMPT: &str = r#"
You are given the full content of a codebase and a filename.

Your task is to extract the relevant portion of code that should go into this file, based on a modular project structure.

Instructions:
- Use the full code context to write the appropriate contents for the file provided.
- The code must be correct, import necessary modules, and function independently when imported.
- Only return the full code for that one file - do not explain, summarize, or list anything else.
- Do not wrap the code in triple backticks or include any extra markers.
- The output should be clean, raw source code, starting from the first line.

Input:
  Filename: `utils/helpers.py`
  Full Project Code: <entire monolithic code>

Output:
def format_date(dt):
    return dt.strftime("%Y-%m-%d")

def validate_email(email):
    return "@" in email
"#;
