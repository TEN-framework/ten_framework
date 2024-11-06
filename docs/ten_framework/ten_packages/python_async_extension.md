# Python Async Extension

A **Python async extension** is a modular component that using Python's `asyncio` framework. If you want to wrap existing Python codes that use `asyncio` into a TEN extension, using the Python async extension is the simplest and most convenient option.

## Example: The Default Python Async Extension

Here’s how a Python async extension is structured:

```python
import asyncio
from ten import AsyncExtension, AsyncTenEnv

class DefaultAsyncExtension(AsyncExtension):
    async def on_configure(self, ten_env: AsyncTenEnv) -> None:
        # Mock async operation, e.g. network, file I/O.
        await asyncio.sleep(0.5)

    async def on_init(self, ten_env: AsyncTenEnv) -> None:
        # Mock async operation, e.g. network, file I/O.
        await asyncio.sleep(0.5)

    async def on_start(self, ten_env: AsyncTenEnv) -> None:
        # Mock async operation, e.g. network, file I/O.
        await asyncio.sleep(0.5)

    async def on_deinit(self, ten_env: AsyncTenEnv) -> None:
        # Mock async operation, e.g. network, file I/O.
        await asyncio.sleep(0.5)

    async def on_cmd(self, ten_env: AsyncTenEnv, cmd: Cmd) -> None:
        cmd_json = cmd.to_json()
        ten_env.log_debug(f"DefaultAsyncExtension on_cmd: {cmd_json}")

        # Mock async operation, e.g. network, file I/O.
        await asyncio.sleep(0.5)

        # Send a new command to other extensions and wait for the result. The
        # result will be returned to the original sender.
        new_cmd = Cmd.create("hello")
        cmd_result = await ten_env.send_cmd(new_cmd)
        ten_env.return_result(cmd_result, cmd)
```

Each method simulates a delay using `await asyncio.sleep()`.

## FAQ

### Aysnc loop for event handling

- Create a queue: use `asyncio.Queue`.
- Create an async task for event handling.

Here is the sample code:

```python
import asyncio
from ten import AsyncExtension, AsyncTenEnv

class DefaultAsyncExtension(AsyncExtension):
    queue = asyncio.Queue()
    loop:asyncio.AbstractEventLoop = None

    async def on_start(self, ten_env: AsyncTenEnv) -> None:
        self.loop = asyncio.get_event_loop()

        self.loop.create_task(self._consume())

    async def on_stop(self, ten_env: AsyncTenEnv) -> None:
        self.queue.put(None)

    async def _consume(self) -> None:
        while True
            try:
                value = await self.queue.get()
                if value is None:
                    self.ten_env.log_info("async loop exit")
                    break

                # Code for processing values retrieved from the queue.

            except Exception as e:
                self.ten_env.log_error(f"Failed to handle {e}")
```

## Conclusion

TEN's Python async extension provide a powerful way to handle long-running tasks asynchronously. By integrating Python’s `asyncio` framework, the extensions ensure that operations such as network calls or file handling are efficient and non-blocking. This makes TEN a great choice for building scalable, modular applications with asynchronous capabilities.
