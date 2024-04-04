pub(crate) const ARCHITECT_SCOPE_PROMPT: &str = r#"
Generate a response tailored for website project descriptions.

Instructions:
- Provide a user request describing the desired functionalities and features for a website project.
- Translate the user request into a structured JSON response listing the necessary components for building the website.
- Ensure that at least one of the boolean flags in the response is set to true to indicate a required feature.
- Use the following JSON format for the response:

{
  "crud": bool, // Indicates if CRUD functionality is needed
  "auth": bool, // Indicates if user authentication (login and logout) is required
  "external": bool // Indicates if integration with external data sources is needed
}

Examples:
1. User Request: "Develop an online marketplace for buying and selling products."
   Response:
   {
     "crud": true,
     "auth": true,
     "external": true
   }

2. User Request: "Create a blog platform for publishing articles and comments."
   Response:
   {
     "crud": true,
     "auth": true,
     "external": false
   }

3. User Request: "Build a portfolio website to showcase my projects and skills."
   Response:
   {
     "crud": false,
     "auth": false,
     "external": false
   }
"#;

pub(crate) const ARCHITECT_ENDPOINTS_PROMPT: &str = r#"
Generate a response focused on selecting external API endpoints for website development.

Instructions:
- Provide a detailed project description outlining the purpose and functionalities required for the website.
- Identify and compile a list of external public API endpoints that align with the project's objectives.
- Prioritize endpoints that do not require API keys for access.
- Format the response as a list of URLs enclosed in square brackets, like so: ["url_1", "url_2", "url_3", ...]

Examples:
1. Project Description: "Create a weather forecast website for global cities."
   Response:
   ["https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_API_KEY", "https://api.weatherapi.com/v1/current.json?key=YOUR_API_KEY&q=New%20York"]

2. Project Description: "Develop a news aggregator platform for top headlines."
   Response:
   ["https://newsapi.org/v2/top-headlines?country=us&apiKey=YOUR_API_KEY", "https://api.currentsapi.services/v1/latest-news"]

3. Project Description: "Build a real-time cryptocurrency price tracker."
   Response:
   ["https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd", "https://api.nomics.com/v1/currencies/ticker?key=YOUR_API_KEY&ids=BTC"]

4. Project Description: "Design a recipe sharing website with ingredient search functionality."
   Response:
   ["https://api.spoonacular.com/recipes/complexSearch?apiKey=YOUR_API_KEY&query=pasta", "https://www.themealdb.com/api/json/v1/1/search.php?s=chicken"]

Ensure the generated URLs correspond closely to the project's requirements and can be seamlessly integrated into the website.
"#;

pub(crate) const ARCHITECT_DIAGRAM_PROMPT: &str = r#"
Generate Python diagram code using the diagrams library based on the provided architecture requirements.

Instructions:
- Provide a detailed architectural diagram outlining the purpose and required functionalities for the system.
- Ensure that the generated diagram code accurately represents the architecture described in the prompt.
- Use the provided examples as templates, adapting them to match the specific requirements.
- Customize the generated code by adding or modifying components as needed to reflect the architecture accurately.
- Consider factors such as services, pods, deployments, replicas, storage, and networking components when generating the diagram.
- Aim for clarity and coherence in the generated diagram code to facilitate understanding and communication.

Examples:
1. User Request: "Generate a Stateful Architecture on Kubernetes."
   Response:
      from diagrams import Cluster, Diagram
      from diagrams.k8s.compute import Pod, StatefulSet
      from diagrams.k8s.network import Service
      from diagrams.k8s.storage import PV, PVC, StorageClass

      with Diagram("Stateful Architecture", show=False):
          with Cluster("Apps"):
              svc = Service("svc")
              sts = StatefulSet("sts")

              apps = []
              for _ in range(3):
                  pod = Pod("pod")
                  pvc = PVC("pvc")
                  pod - sts - pvc
                  apps.append(svc >> pod >> pvc)

          apps << PV("pv") << StorageClass("sc")

2. User Request: "Generate an Exposed Pod with 3 Replicas on Kubernetes."
   Response:
      from diagrams import Diagram
      from diagrams.k8s.clusterconfig import HPA
      from diagrams.k8s.compute import Deployment, Pod, ReplicaSet
      from diagrams.k8s.network import Ingress, Service

      with Diagram("Exposed Pod with 3 Replicas", show=False):
          net = Ingress("domain.com") >> Service("svc")
          net >> [Pod("pod1"),
                  Pod("pod2"),
                  Pod("pod3")] << ReplicaSet("rs") << Deployment("dp") << HPA("hpa")

Ensure the generated code closely corresponds to the user prompt and project requirements.
"#;
