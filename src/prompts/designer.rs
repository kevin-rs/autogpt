pub(crate) const WEB_DESIGNER_PROMPT: &str = r#"
Your task is to describe the web design layout elements based on the provided image. Describe each ui element from left to right, top to bottom.

Instructions:
- The image represents a visual representation of a web design.
- Your goal is to generate a description of the web design depicted in the image.
- The description should be concise and capture the key elements of the design.
- Ensure that your description begins with the phrase "The web design features..." followed by a description of the design elements.
- Avoid including any interpretation or subjective opinions in your description.

Example:
  Image: [Attach the image here]
  Description: "The web design features a modern layout with a prominent hero section showcasing a beach scene. The navigation bar is minimalist, and the content sections are organized with clean typography and ample whitespace."
"#;

pub(crate) const STABILITY_PROMPT: &str = r#"
Your task is to generate a web design based on the provided description.

Instructions:
- The user has provided a description of their desired web design.
- Your goal is to use the Stable Diffusion model to generate a visual representation of the described web design.
- The description should guide the generation of the web design elements.
- Ensure that the generated design reflects the key elements mentioned in the description.
- The output should be a visual representation of the web design described.

Example:
  Description: "A minimalist layout with a clean interface showing the forecast for the next week, with options to switch between Celsius and Fahrenheit."
  Output: [your web design image goes here]

  Description: "An interactive map displaying real-time weather data for different locations, with customizable layers for precipitation, temperature, and wind speed."
  Output: [your web design image goes here]]
"#;
