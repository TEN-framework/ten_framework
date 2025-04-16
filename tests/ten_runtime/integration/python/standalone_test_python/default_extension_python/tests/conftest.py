#
# This file is part of TEN Framework, an open source project.
# Licensed under the Apache License, Version 2.0.
# See the LICENSE file for more information.
#
import threading
import pytest
from ten import (
    App,
    TenEnv,
)


class FakeApp(App):
    def __init__(self):
        super().__init__()
        self.event: threading.Event | None = None

    def on_init(self, ten_env: TenEnv) -> None:
        assert self.event
        self.event.set()

        ten_env.on_init_done()


class FakeAppCtx:
    def __init__(self, event: threading.Event):
        self.fake_app: FakeApp | None = None
        self.event = event


def run_fake_app(fake_app_ctx: FakeAppCtx):
    app = FakeApp()
    app.event = fake_app_ctx.event
    fake_app_ctx.fake_app = app
    app.run(False)


@pytest.fixture(scope="session", autouse=True)
def global_setup_and_teardown():
    event = threading.Event()
    fake_app_ctx = FakeAppCtx(event)

    fake_app_thread = threading.Thread(
        target=run_fake_app, args=(fake_app_ctx,)
    )
    fake_app_thread.start()

    event.wait()

    assert fake_app_ctx.fake_app is not None

    # Yield control to the test; after the test execution is complete, continue
    # with the teardown process.
    yield

    # Teardown part.
    fake_app_ctx.fake_app.close()
    fake_app_thread.join()
