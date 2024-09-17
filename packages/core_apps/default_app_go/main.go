//
// This file is part of the TEN Framework project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

package main

import (
	"fmt"

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
}
