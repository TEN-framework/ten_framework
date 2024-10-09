---
layout:
  title:
    visible: true
  description:
    visible: false
  tableOfContents:
    visible: true
  outline:
    visible: true
  pagination:
    visible: true
---

# How to run local model in Python extensions

In the 'TEN' world, extensions can not only leverage third-party AI model
services to implement functionalities but also run AI models locally which
reduces the latency and cost of the service. In this tutorial, we will introduce
how to run a local model in a Python extension, and how to interact with the
model in the extension.

## Step 1: Ensure Hardware Requirements are Met

Before you begin running the AI model locally, it is essential to confirm that your hardware meets the necessary requirements. The key components to check are the CPU, GPU, and memory. The AI model you plan to run locally may have specific hardware requirements, so it is important to verify that your hardware meets these requirements.

## Step 2: Install Required Software and Dependencies

After ensuring that your hardware meets the necessary requirements, the next step is to install the required software and dependencies. Following these guidelines:

1. **Operating System**: Ensure that your operating system is compatible with the model. Most AI frameworks support Windows, macOS, and Linux, but specific versions may be required.
2. **Python Version**: As the ten python runtime is bound with a specific Python version, ensure that the Python version you are using is compatible with the model and the inference engine.
3. **Install Required Libraries**: Depending on the AI model, you will need to install specific libraries. Commonlly used libraries include:

   - TensorFlow
   - PyTorch
   - Numpy
   - vllm
   - ...

   You can install these libraries in your Python environment and save the requirements in a `requirements.txt` file under the extension root directory.

4. **Download the Model**: Download the AI model you plan to run locally.

## Step 3: Implement your Python Extension

Below is a sample on how to implement a simple text generation in a Python extension using vllm inference engine.

First, load the local model during the extension initialization:

```python
from ten import (
    Extension,
    TenEnv,
    Cmd,
    CmdResult,
)
from vllm import LLM

class TextGenerationExtension(Extension):
    def on_init(self, ten_env: TenEnv) -> None:
        self.llm = LLM(model="<model_path>")

        ten_env.on_init_done()
```

Next, implement the `on_cmd`/`on_data` method to handle the command/data:

```python
    def on_cmd(self, ten_env: TenEnv, cmd: Cmd) -> None:
        prompt = cmd.get_property_string("prompt")

        outputs = self.llm.generate(prompt)
        generated_text = outputs[0].outputs[0].text

        cmd_result = CmdResult.create(StatusCode.OK)
        cmd_result.set_property_string("result", generated_text)
        ten_env.return_result(cmd_result, cmd)
```

Above, in `on_cmd` method, 'prompt' is retrieved from the property of the command, then the model generates the text based on the prompt, and the generated text is returned as the result of the command.

Besides text generation, you can also implement other functionalities like image recognition (image in, text out), speech-to-text (audio in, text out), etc., using the local model.

## Step 4: Unload the model

Unload the model during the extension cleanup:

```python
import gc
import torch

class TextGenerationExtension(Extension):
    ...

    def on_deinit(self, ten_env: TenEnv) -> None:
        del self.llm
        gc.collect()
        torch.cuda.empty_cache()
        torch.distributed.destroy_process_group()
        print("Successfully delete the llm pipeline and free the GPU memory!")

        ten_env.on_deinit_done()
```

## Summary

In the TEN Python extension, running a local model is almost indistinguishable from a native Python development experience. By placing model loading and unloading in the appropriate extension lifecycle methods and designing the extension's input and output well, you can easily run a local model in a Python extension and interact with it.
