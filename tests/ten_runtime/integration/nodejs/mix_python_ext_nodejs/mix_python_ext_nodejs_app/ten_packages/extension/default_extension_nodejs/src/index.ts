//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
import {
  Addon,
  RegisterAddonAsExtension,
  Extension,
  TenEnv,
  Cmd,
  Data,
  CmdResult,
  StatusCode,
} from "ten-runtime-nodejs";

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

class DefaultExtension extends Extension {
  constructor(name: string) {
    super(name);
  }

  async onConfigure(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onConfigure");
  }

  async onInit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onInit");
  }

  async onStart(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onStart");

    const testData = Data.Create("testData");
    testData.allocBuf(10);
    let buf = testData.lockBuf();

    let view = new Uint8Array(buf);
    view[0] = 1;
    view[1] = 2;
    view[2] = 3;
    testData.unlockBuf(buf);

    let copiedBuf = testData.getBuf();
    let copiedView = new Uint8Array(copiedBuf);
    assert(copiedView[0] === 1, "copiedView[0] incorrect");
    assert(copiedView[1] === 2, "copiedView[1] incorrect");
    assert(copiedView[2] === 3, "copiedView[2] incorrect");
  }

  async onStop(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onStop");
  }

  async onDeinit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtension onDeinit");
  }

  async onCmd(tenEnv: TenEnv, cmd: Cmd): Promise<void> {
    tenEnv.logDebug("DefaultExtension onCmd");

    const cmdName = cmd.getName();
    tenEnv.logVerbose("cmdName:" + cmdName);

    const testCmd = Cmd.Create("test");
    const [result, _] = await tenEnv.sendCmd(testCmd);
    assert(result !== null, "result is null");

    const detailJson = result.getPropertyToJson("detail");

    tenEnv.logInfo("received result detail:" + detailJson);

    const cmdResult = CmdResult.Create(StatusCode.OK);
    cmdResult.setPropertyString("detail", detailJson);
    tenEnv.returnResult(cmdResult, cmd);
  }
}

@RegisterAddonAsExtension("default_extension_nodejs")
class DefaultExtensionAddon extends Addon {
  async onInit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtensionAddon onInit");
  }

  async onCreateInstance(
    _tenEnv: TenEnv,
    instanceName: string
  ): Promise<Extension> {
    return new DefaultExtension(instanceName);
  }

  async onDeinit(_tenEnv: TenEnv): Promise<void> {
    console.log("DefaultExtensionAddon onDeinit");
  }
}
