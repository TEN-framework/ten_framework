//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package default_extension_go

import (
	"fmt"
	"sync"
	"sync/atomic"

	"ten_framework/ten"
)

const concurrency = 100

type extensionA struct {
	ten.DefaultExtension
}

func newExtensionA(name string) ten.Extension {
	return &extensionA{}
}

func (p *extensionA) OnStart(tenEnv ten.TenEnv) {
	go func() {
		var wg sync.WaitGroup
		var counter int32

		wg.Add(concurrency)

		for i := int64(0); i < concurrency; i++ {
			go func(i int64) {
				defer wg.Done()

				err := tenEnv.SetProperty(
					fmt.Sprintf("prop_%d", i),
					i,
				)
				if err != nil {
					fmt.Printf("Error in goroutine %d: %v\n", i, err)
					panic("Should not happen")
				}

				read_back_value, err := tenEnv.GetPropertyInt64(
					fmt.Sprintf("prop_%d", i),
				)
				if err != nil {
					fmt.Printf("Error in goroutine %d: %v\n", i, err)
					panic("Should not happen")
				}

				if read_back_value != i {
					fmt.Printf(
						"Error in goroutine %d: %v\n",
						i,
						read_back_value,
					)
					panic("Should not happen")
				}

				if atomic.AddInt32(&counter, 1)%1 == 0 {
					fmt.Printf("extension_a %d goroutines completed\n", counter)
				}
			}(i % 100)
		}

		wg.Wait()
		tenEnv.OnStartDone()
	}()

	fmt.Println("extension_a set empty_string")
	if err := tenEnv.SetPropertyString("empty_string", ""); err != nil {
		panic("Should not happen")
	}

	fmt.Println("extension_a get empty_string")
	if em, err := tenEnv.GetPropertyString("empty_string"); err != nil ||
		em != "" {
		fmt.Printf(
			"Error %v\n",
			em,
		)
		panic("Should not happen")
	}

	fmt.Println("extension_a set test_string")
	if err := tenEnv.SetPropertyString("test_string", "test"); err != nil {
		panic("Should not happen")
	}

	fmt.Println("extension_a get test_string")
	if test_string, err := tenEnv.GetPropertyString("test_string"); err != nil ||
		test_string != "test" {
		panic("Should not happen")
	}

	fmt.Println("extension_a set test_bytes")
	if err := tenEnv.SetPropertyBytes("test_bytes", []byte("test")); err != nil {
		panic("Should not happen")
	}

	fmt.Println("extension_a get test_bytes")
	if test_bytes, err := tenEnv.GetPropertyBytes("test_bytes"); err != nil ||
		string(test_bytes) != "test" {
		panic("Should not happen")
	}
}

func (p *extensionA) OnCmd(
	tenEnv ten.TenEnv,
	cmd ten.Cmd,
) {
	go func() {
		fmt.Println("extensionA OnCmd")

		var propLock sync.Mutex

		cmdB, _ := ten.NewCmd("B")
		var count uint32 = 0

		done := make(chan struct{}, 1)
		defer close(done)

		for i := 0; i < concurrency; i++ {
			go func(i int) {
				propLock.Lock()

				// The SetProperty is not goroutine safety.
				err := cmdB.SetProperty(fmt.Sprintf("prop_%d", i), i)
				propLock.Unlock()

				if err != nil {
					panic("should not happen")
				}

				cmdName, _ := cmdB.GetName()
				if cmdName != "B" {
					panic("should not happen")
				}

				if atomic.AddUint32(&count, 1) == concurrency {
					done <- struct{}{}
				}
			}(i % 100)
		}
		<-done

		// Set an empty string to cmd is permitted.
		if err := cmdB.SetPropertyString("empty_string", ""); err != nil {
			panic("Should not happen.")
		}

		// Set an empty bytes to cmd is not permitted.
		if err := cmdB.SetPropertyBytes("empty_bytes", []byte{}); err == nil {
			panic("Should not happen.")
		}

		some_bytes := []byte{1, 2, 3}
		if err := cmdB.SetPropertyBytes("some_bytes", some_bytes); err != nil {
			panic("Should not happen.")
		}

		tenEnv.SendCmd(cmdB, func(r ten.TenEnv, cs ten.CmdResult, e error) {
			detail, err := cs.GetPropertyString("detail")
			if err != nil {
				cmdResult, _ := ten.NewCmdResult(ten.StatusCodeError, cmd)
				cmdResult.SetPropertyString("detail", err.Error())
				r.ReturnResult(cmdResult, nil)
				return
			}

			if detail != "this is extensionB." {
				cmdResult, _ := ten.NewCmdResult(ten.StatusCodeError, cmd)
				cmdResult.SetPropertyString("detail", "wrong detail")
				r.ReturnResult(cmdResult, nil)
				return
			}

			password, err := cs.GetPropertyString("password")
			if err != nil {
				cmdResult, _ := ten.NewCmdResult(ten.StatusCodeError, cmd)
				cmdResult.SetPropertyString("detail", err.Error())
				r.ReturnResult(cmdResult, nil)
				return
			}

			cmdResult, _ := ten.NewCmdResult(ten.StatusCodeOk, cmd)
			cmdResult.SetPropertyString("detail", password)
			r.ReturnResult(cmdResult, nil)
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
