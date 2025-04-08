# Pinecone Account and API Key Setup

[Pinecone](https://www.pinecone.io/) is a vector database that enables you to store, search, and manage high-dimensional vector data at scale. To get started with Pinecone and configure your API key, follow these steps:

## 1. Create a Pinecone Account and Obtain API Key

1. **Visit the Pinecone Website:** Go to the official Pinecone website at [https://www.pinecone.io/](https://www.pinecone.io/) to create a Pinecone account if you don't already have one.

1. **Registration Process:** Complete the registration process by filling in the necessary information. After registering, you will be directed to the Pinecone dashboard.

1. **Obtain API Key:** After successfully registration, you will obtain a new API key that will be used to authenticate autogpt with Pinecone.
   ![API Key](https://github.com/user-attachments/assets/ff58f0df-2727-43c4-a866-43a1a4cff616)

1. **Copy the API Key:** Once the API key is generated, copy it to your clipboard. You will need to use this API key to authenticate autogpt with Pinecone.

1. **Create an Index:** Once your project is created, you can create an index. An index is where your vector data will be stored. Select the appropriate index configuration for your project.
   ![Create Index](https://github.com/user-attachments/assets/904838cf-e4d7-4e0d-8ee4-6185fcf993e6)

## 2. Set Your Pinecone API Key and Index URL in AutoGPT

Now that you have your Pinecone API key, you need to set it in autogpt:

1. **Add the Pinecone API Key and Index URL:** In your configuration file, add the following environment variables, replacing `<Your_Pinecone_API_Key>` with the actual API key you obtained from the Pinecone dashboard, and `<Your_Pinecone_Index_URL>` with the appropriate index URL for your Pinecone instance:
   ```toml
   PINECONE_API_KEY=<Your_Pinecone_API_Key>
   PINECONE_INDEX_URL=<Your_Pinecone_Index_URL>
   ```

By setting the `PINECONE_API_KEY` and `PINECONE_INDEX_URL` environment variables, autogpt will be able to authenticate and interact with the Pinecone API.

## 3. Notes

- **Each Communication gets stored as a vector in Pinecone:** The content of each agent's communication messages is stored as a vector in Pinecone.
- **Embedding Process:** The content field of the communication will be embedded into a vector. To do this, autogpt uses a way to convert text into vector representations, such as through OpenAI, Gemini, or a local embedding model.
- **Namespace for Vectors:** Vectors are stored in a specific namespace that is based on the agent's unique ID. The format of the namespace will be `"agent-<agent_id>"`. This allows autogpt to store and organize vectors associated with different agents separately.

![DB Data](https://github.com/user-attachments/assets/bda24b91-e7d9-47b0-8e55-a11479ee5eb6)
