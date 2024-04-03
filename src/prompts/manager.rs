pub(crate) const MANAGER_PROMPT: &str = r#"
Your task is to translate user requests into concise project steps given the project goal and the role of the agent (e.g. frontend, backend, etc).

Instructions:
- The user will provide an input, and your task is to generate summarized project goal steps, each as a bullet point.
- Develop the logic to accurately process user input and produce concise project goals.
- Ensure that your output does include the selected programming language, web framework in the project's bullet points.

Example 1:
  User Request: Project Goal: "Develop a platform for online courses with video streaming, quizzes, and progress tracking.", Agent Role: "frontend", programming language: "JavaScript", framework: "React.js"
  Output: 
  - Using React.js, build a website in JavaScript that provides online courses with video streaming.
  - Step 1: Define the basic layout and structure of the website.
  - Step 2: Implement video streaming functionality using appropriate libraries or APIs.

Example 2:
  User Request: Project Goal: "Create a mobile application for task management with calendar integration and notification features.", Agent Role: "backend", programming language: "Python", framework: "Django"
  Output: 
  - Utilizing Django, develop a backend in Python for a task management mobile app with calendar integration and notification features.
  - Step 1: Set up database models for tasks and user data.
  - Step 2: Implement calendar integration and notification services using Django's built-in features or external libraries.

Example 3:
  User Request: Project Goal: "Build an e-commerce website with product catalog, user authentication, and payment gateway integration.", Agent Role: "full-stack", programming language: "JavaScript", framework: "Node.js"
  Output: 
  - With Node.js, create a full-stack JavaScript application for an e-commerce website featuring a product catalog, user authentication, and payment gateway integration.
  - Step 1: Design and develop the front-end interface for product catalog and user authentication.
  - Step 2: Integrate payment gateway APIs and implement secure transaction handling on the backend.

Example 4:
  User Request: Project Goal: "Develop a chat application with real-time messaging and file sharing functionality.", Agent Role: "frontend", programming language: "TypeScript", framework: "Vue.js"
  Output: 
  - Using Vue.js, design a frontend in TypeScript for a chat application offering real-time messaging and file sharing functionality.
  - Step 1: Set up WebSocket connection for real-time messaging between users.
  - Step 2: Implement file upload/download functionality and integrate it with the chat interface.

Example 5:
  User Request: Project Goal: "Design a social media platform with profiles, posts, comments, and messaging features.", Agent Role: "backend", programming language: "Java", framework: "Spring Boot"
  Output: 
  - Implementing Spring Boot, construct a backend in Java for a social media platform encompassing profiles, posts, comments, and messaging features.
  - Step 1: Define database schemas for user profiles, posts, comments, and messages.
  - Step 2: Develop RESTful APIs for CRUD operations on user data, posts, and comments, as well as messaging functionalities.
"#;

pub(crate) const LANGUAGE_MANAGER_PROMPT: &str = r#"
Your task is to extract the programming language mentioned in the user requests.

Instructions:
- The user will provide an input describing a project goal along with the programming language used.
- Your task is to identify and output the programming language mentioned in the input.
- Ensure that your output does not include any commentary other than the programming language.

Example 1:
  User Request: "Develop a platform for online courses with video streaming, quizzes, and progress tracking using Python"
  Output: Python

Example 2:
  User Request: "Create a mobile application for task management with calendar integration and notification features using JavaScript"
  Output: JavaScript

Example 3:
  User Request: "Build a data analysis tool for financial forecasting using R programming language"
  Output: R

Example 4:
  User Request: "Implement backend services for a social media platform using Java"
  Output: Java
"#;

pub(crate) const FRAMEWORK_MANAGER_PROMPT: &str = r#"
Your task is to extract the web framework mentioned in the user requests.

Instructions:
- The user will provide an input describing a project goal along with the web framework used.
- Your task is to identify and output the web framework mentioned in the input.
- Ensure that your output does not include any commentary other than the web framework.

Example 1:
  User Request: "Develop a platform for online courses with video streaming, quizzes, and progress tracking using FastAPI web framework"
  Output: FastAPI framework

Example 2:
  User Request: "Create a mobile application for task management with calendar integration and notification features using the React.js web framework"
  Output: React.js framework

Example 3:
  User Request: "Build a blogging website with Django framework"
  Output: Django framework

Example 4:
  User Request: "Develop a real-time chat application using Axum framework"
  Output: Axum framework
"#;
