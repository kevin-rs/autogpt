pub(crate) const FRONTEND_CODE_PROMPT: &str = r#"
Your task is to generate frontend code for a web application using the selected framework.

Instructions:
- The user will provide a project description and a code template for a frontend web application.
- The frontend code provided is only an example. Modify it as needed to match the project description.
- Write components and functions that make sense for the user's request if required.
- You can use the selected framework and any other necessary libraries.
- You should only output the code, nothing else.
- You should remove all backticks surrounding the source code. Remove the first and last lines(remove "```").

Example:

Input:
  Project Description: "Build a simple todo list web application."
  Code Template: "<div>{ for tasks.iter().map(|task| html! { <li>{ task }</li> }) }</div>"

Output:
<div>
    <ul>
        <li>Task 1</li>
        <li>Task 2</li>
        <li>Task 3</li>
    </ul>
</div>
"#;

pub(crate) const IMPROVED_FRONTEND_CODE_PROMPT: &str = r#"
Your task is to improve the provided frontend code for a web application using the selected framework.

Instructions:
- The user will provide a project description and a code template for a frontend web application.
- Task:
  1. Fix any bugs in the code and add minor additional functionality.
  2. Ensure compliance with all frontend requirements specified in the project description. Add any missing features.
  3. Write the code without any commentary.
- You can use the selected framework and any other necessary libraries.
- You should only output the code, nothing else.
"#;

pub(crate) const FIX_CODE_PROMPT: &str = r#"
Your task is to fix the code with removed bugs.

Instructions:
- The user will provide a broken code and the identified errors or bugs.
- Your task is to fix the bugs in the code.
- You should only output the new and improved code, without any commentary.
"#;
