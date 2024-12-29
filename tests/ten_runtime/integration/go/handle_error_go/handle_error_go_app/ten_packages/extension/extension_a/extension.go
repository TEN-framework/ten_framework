//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to https://github.com/TEN-framework/ten_framework/LICENSE for more
// information.
//

package default_extension_go

import (
	"fmt"

	"ten_framework/ten"
)

type extensionA struct {
	ten.DefaultExtension
}

func newExtensionA(name string) ten.Extension {
	return &extensionA{}
}

func (p *extensionA) OnCmd(
	tenEnv ten.TenEnv,
	cmd ten.Cmd,
) {
	go func() {
		fmt.Println("extensionA OnCmd")

		cmdB, _ := ten.NewCmd("B")
		tenEnv.SendCmd(cmdB, func(r ten.TenEnv, cs ten.CmdResult, e error) {
			detail, err := cs.GetPropertyString("detail")
			if err != nil {
				cmdResult, _ := ten.NewCmdResult(ten.StatusCodeError)
				cmdResult.SetPropertyString("detail", err.Error())
				r.ReturnResult(cmdResult, cmd, nil)
				return
			}

			// Try to get nonexistent property.
			_, err = tenEnv.GetPropertyString("agora")
			if err != nil {
				fmt.Println("getProp agora error, ", err)
			}

			_, err = tenEnv.GetPropertyString("agora")
			if err == nil {
				panic("should not happen")
			}

			fmt.Println("GetPropertyString error, ", err)

			statusCode, _ := cs.GetStatusCode()
			cmdResult, _ := ten.NewCmdResult(statusCode)
			cmdResult.SetPropertyString("detail", detail)
			r.ReturnResult(cmdResult, cmd, nil)
		})
	}()
}

func init() {
	// Register addon
	err := ten.RegisterAddonAsExtension(
		"extension_a",
		ten.NewDefaultExtensionAddon(newExtensionA),
	)
	if err != nil {
		fmt.Println("register addon failed", err)
	}
}
