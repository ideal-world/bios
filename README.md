**WIP**

# BIOS(Business Informatization OS)

[中文](README_CN.md)   [English](README.md)

> For computers, BIOS (Basic Input/Output System) is the foundation for loading and running the operating system. It abstracts hardware access and serves as a critical link between hardware and software.
> For enterprises, we also hope to have a similar system that provides basic capabilities for upper-level business applications and abstracts the access of mainstream cloud services, providing a consistent access interface.

In a nutshell, BIOS is a **stable**, **secure**, **lightweight**, and **extensible** technology platform that empowers enterprises to achieve digital transformation.

## Ability Layering

![architecture](architecture.png)

The vast majority of `BIOS` functionalities are written in the ``Rust`` language and rely on the ``Tardis``(https://github.com/ideal-world/tardis) framework maintained by the same group of contributors.

From the bottom to the top, ``BIOS`` is divided into five layers: **RBUM**, **SPI**, **Middleware**, **Support**, and **FaaS**.

* **RBUM** (Resource-Based Unified Model) is a unified model based on resources. The model provides upper-level operations with `unified basic operations, unified credential management, and unified access control`.

* **SPI** (Service Provider Interface) provides abstractions for commonly used basic operations to adapt to mainstream middleware/cloud services. For example, we provide the `full-text search` operation, which adapts to `PostgreSql` and `ElasticSearch` and can be further extended to other implementations.

* **Middleware** provides some commonly used middleware. Different from the `SPI` layer, these middleware do not consider adaptability, thus gaining greater flexibility and freedom, and can be used to build some special features. For example, we provide the `event service`, which is based on the `Websocket` protocol and implements event penetration between the front and back ends and between the back and back ends.

* **Support** is used to provide some complex domain services. Different from the `Middleware` layer, these supporting services aggregate the capabilities of `SPI` and `Middleware` to form more complex business-oriented services.

* **FaaS** is used to implement the construction of general business applications with simple front-end technologies.

All of these layers are optional. They are `libraries` and do not contain executable `services`. We use a special `aggregation service` layer to aggregate different capabilities into the required services. In actual use, we can select the required capabilities to build customized services that meet our own needs.

In terms of gateway selection, we support the self-developed gateway named `SpaceGate` ([https://github.com/ideal-world/spacegate](https://github.com/ideal-world/spacegate)) by default to better integrate with `BIOS`.


## Directory Structure

```
|-- backend
  |-- basic                 Basic operation module, including common logic of RBUM and SPI
  |-- spi                   SPI layer
  |-- middleware            Middleware layer
  |-- support               Support layer
  |-- faas                  FaaS layer
  |-- services              Aggregation service layer
  |-- gateway               Gateway adaptation layer
    |-- spacegate-plugins   Customized plugins for SpaceGate gateway
|-- frontend
  |-- console               Console front-end
  |-- sdks                  Interface encapsulation and operation client
|-- examples                Usage examples
|-- docs                    Documentation
```