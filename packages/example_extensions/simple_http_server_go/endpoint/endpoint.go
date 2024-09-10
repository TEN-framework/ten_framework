// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
package endpoint

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"ten_framework/ten"
)

type Endpoint struct {
	server *http.Server
	tenEnv ten.TenEnv // The communication bridge of this extension.
}

func NewEndpoint(tenEnv ten.TenEnv) *Endpoint {
	return &Endpoint{
		tenEnv: tenEnv,
	}
}

func (s *Endpoint) defaultHandler(
	writer http.ResponseWriter,
	request *http.Request,
) {
	switch request.URL.Path {
	case "/health":
		writer.WriteHeader(http.StatusOK)
	default:
		statusChan := make(chan ten.CmdResult, 1)
		cmd, _ := ten.NewCmd("demo")

		s.tenEnv.SendCmd(cmd, func(tenEnv ten.TenEnv, cmdStatus ten.CmdResult) {
			statusChan <- cmdStatus
		})

		select {
		case status := <-statusChan:
			writer.WriteHeader(http.StatusOK)

			detail, _ := status.GetPropertyString("detail")
			writer.Write([]byte(detail))
		}
	}
}

func (e *Endpoint) Start() {
	mux := http.NewServeMux()
	mux.HandleFunc("/", e.defaultHandler)

	e.server = &http.Server{
		Addr:    ":8001",
		Handler: mux,
	}

	go func() {
		if err := e.server.ListenAndServe(); err != nil {
			if err != http.ErrServerClosed {
				panic(err)
			}
		}
	}()

	go func() {
		// Check if the server is ready.
		for {
			resp, err := http.Get("http://127.0.0.1:8001/health")
			if err != nil {
				continue
			}

			defer resp.Body.Close()

			if resp.StatusCode == 200 {
				break
			}

			time.Sleep(50 * time.Millisecond)
		}

		fmt.Println("http server starts.")

		e.tenEnv.OnStartDone()
	}()
}

func (s *Endpoint) Stop() {
	if s.server != nil {
		s.server.Shutdown(context.Background())
	}
}
