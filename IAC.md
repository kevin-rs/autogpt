# RFC: IAC Protocol for Inter and Intra Agents Communications

![iac-seq-diag](https://github.com/user-attachments/assets/555534b1-84ca-4efc-ac9d-553e4a979d48)

## Introduction

The **IAC (Inter and Intra Agents Communication)** protocol is a novel protocol in the AutoGPT ecosystem, designed to manage communication between the orchestrator and agents. It is inspired by operating system Inter-Process Communication (IPC) protocols but adapted to the needs of a distributed, agent-based system like AutoGPT. This document provides a detailed explanation of the protocol, its design choices, and its advantages over other methods of communication.

## Background

AutoGPT is a system where multiple agents work together to accomplish tasks. These agents communicate with each other and the orchestrator. The communication needs to be secure, fast, and efficient. Existing protocols for message passing and communication, such as REST APIs or simple message queues, often lack the performance and security needed for real-time, high-performance interactions among agents.

### Protocol Design Philosophy

The **IAC protocol** was designed with the following core principles:

1. **Efficiency**: Fast, compact communication with minimal overhead.
1. **Security**: TLS encryption over TCP for secure communication between agents and the orchestrator.
1. **Scalability**: A protocol capable of scaling as the number of agents increases.
1. **Flexibility**: Capable of supporting both inter-agent and intra-agent communication.

## Protocol Overview

### Communication Model

The communication model of AutoGPT is divided into two parts:

1. **Intra-Agent Communication**: Communication between different agents processes that might reside on the same machine.
1. **Inter-Agent Communication**: Communication between agents that are distributed across different machines.

The **IAC protocol** uses **Protocol Buffers (Protobuf)** as the serialization format for messages. This binary format is compact and efficient for low latency and high throughput. Messages are sent over a **TLS-encrypted TCP connection** between agents and the orchestrator.

### Message Structure

Messages are structured using Protobuf to ensure efficient serialization/deserialization. Each message contains the following fields:

1. **from**: The sender of the message (either an agent or the orchestrator).
1. **to**: The recipient of the message (either an agent or the orchestrator).
1. **msg_type**: The type of the message, such as "create", "terminate", "run", etc.
1. **payload_json**: A JSON string carrying the task-specific data.
1. **auth_token**: A token used to authenticate the sender and recipient.

### TLS over TCP

Communication between agents and the orchestrator is done over a secure **TLS-encrypted TCP** connection. This ensures the confidentiality and integrity of the data exchanged. The use of **TLS** provides authentication and protection against eavesdropping and agent-in-the-middle attacks.

## Protocol Flow

### Establishing a Connection

1. **Orchestrator Setup**: The orchestrator starts up and listens on a specified TCP address.
1. **Agent Connection**: Each agent establishes a TLS connection to the orchestrator.
1. **Handshake**: The orchestrator and agent perform a TLS handshake to establish a secure communication channel.
1. **Message Exchange**: Once the connection is established, messages are exchanged between the orchestrator and agents.

### Message Flow

The orchestrator and agents communicate using a request-response model. A typical message flow looks like this:

1. The orchestrator sends a message to the agent (e.g., to execute a task).
1. The agent processes the request and sends a response back to the orchestrator.
1. The orchestrator may initiate further communication based on the agent's response.

## Advantages of the IAC Protocol

1. **Compact and Efficient**: Using **Protobuf** ensures that the data sent between agents and the orchestrator is serialized efficiently.
1. **Security**: **TLS over TCP** ensures the confidentiality and integrity of the communication, protecting against various network-based attacks.
1. **Real-Time Communication**: The connection-oriented nature of **TCP** ensures that agents and orchestrators can maintain a persistent connection, enabling low-latency, real-time communication.
1. **Inspired by IPC**: By drawing inspiration from **Operating System IPC** protocols, the IAC protocol brings a familiar, efficient, and well-understood model to the world of agent-based systems.
1. **Scalability**: The protocol supports scalability, allowing multiple agents to be added to the system with minimal changes. It can scale efficiently across machines.

## Conclusion

The **IAC protocol** provides an efficient, secure, and scalable communication mechanism for the AutoGPT ecosystem. By utilizing **Protocol Buffers (Protobuf)** for message serialization and **TLS over TCP** for secure connections, the protocol ensures high-performance communication that can scale with the number of agents and orchestrators. Drawing inspiration from traditional **IPC** protocols, **IAC** offers a familiar and robust solution for inter-agent communication.

## Future Work

1. **Optimizations for Low-Bandwidth Networks**: Further compression techniques can be implemented to reduce data usage on slower networks.
1. **Fault Tolerance**: Implementing retry mechanisms and graceful error handling to improve the resilience of the protocol in case of network failures.
