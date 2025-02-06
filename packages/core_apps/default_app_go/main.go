//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

package main

// To detect memory leaks with ASan, need to enable the following cgo code.

// void __lsan_do_leak_check(void);
// import "C"

import (
	"fmt"
	"runtime"
	"runtime/debug"
	"time"

	"ten_framework/ten"
)

type defaultApp struct {
	ten.DefaultApp
}

func (p *defaultApp) OnInit(tenEnv ten.TenEnv) {
	tenEnv.LogDebug("onInit")
	tenEnv.OnInitDone()
}

func (p *defaultApp) OnDeinit(tenEnv ten.TenEnv) {
	tenEnv.LogDebug("onDeinit")
	tenEnv.OnDeinitDone()
}

func main() {
	// test app
	app, err := ten.NewApp(&defaultApp{})
	if err != nil {
		fmt.Println("Failed to create app.")
	}

	app.Run(true)
	app.Wait()

	// Explicitly trigger GC to increase the likelihood of finalizer execution.
	debug.FreeOSMemory()
	runtime.GC()

	// Wait for a short period to give the GC time to run.
	runtime.Gosched()
	time.Sleep(3 * time.Second)

	// To detect memory leaks with ASan, need to enable the following cgo code.
	// C.__lsan_do_leak_check()
}
