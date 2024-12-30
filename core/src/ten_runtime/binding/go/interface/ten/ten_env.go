//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package ten

//#include "ten_env.h"
import "C"

import (
	"runtime"
	"strings"
	"unsafe"
)

type (
	// ResultHandler is a function type that represents a handler for the result
	// of a command.
	ResultHandler func(TenEnv, CmdResult, error)

	// ErrorHandler is a function type that represents a handler for errors of a
	// non-command type message.
	ErrorHandler func(TenEnv, error)
)

// TenEnv represents the interface for the TEN (Run Time Environment) component.
type TenEnv interface {
	SendCmd(cmd Cmd, handler ResultHandler) error
	SendData(data Data, handler ErrorHandler) error
	SendVideoFrame(videoFrame VideoFrame, handler ErrorHandler) error
	SendAudioFrame(audioFrame AudioFrame, handler ErrorHandler) error

	ReturnResult(result CmdResult, cmd Cmd, handler ErrorHandler) error
	ReturnResultDirectly(result CmdResult, handler ErrorHandler) error

	OnConfigureDone() error
	OnInitDone() error
	OnStartDone() error
	OnStopDone() error
	OnDeinitDone() error
	OnCreateInstanceDone(instance any, context uintptr) error

	iProperty
	InitPropertyFromJSONBytes(value []byte) error

	LogVerbose(msg string)
	LogDebug(msg string)
	LogInfo(msg string)
	LogWarn(msg string)
	LogError(msg string)
	LogFatal(msg string)

	// Private functions.
	postSyncJob(payload job) any
	postAsyncJob(payload job) any
	logInternal(level LogLevel, msg string, skip int)
}

// Making a compile-time assertion which indicates that if 'ten' type doesn't
// implement all the methods of the 'Ten' interface, the code will fail to
// compile, and you'll get an error. This is a way to ensure at compile time
// that a certain type satisfies an interface, even if you don't use that type
// to call the interface's methods in your code.
//
// In simpler terms, it's a way to say: "Make sure that the 'ten' type
// implements the 'Ten' interface. I don't need to use this check anywhere else
// in my code; I just want to make sure it's true." If 'ten' doesn't implement
// Ten, you'll know as soon as you try to compile.
var (
	_ TenEnv = new(tenEnv)
)

type tenAttachTo uint8

const (
	tenAttachToInvalid tenAttachTo = iota
	tenAttachToExtension
	tenAttachToExtensionGroup
	tenAttachToApp
	tenAttachToAddon
)

type tenEnv struct {
	baseTenObject[C.uintptr_t]

	attachToType tenAttachTo
	attachTo     unsafe.Pointer
}

func (p *tenEnv) attachToExtension(ext *extension) {
	if p.attachToType != tenAttachToInvalid {
		panic("The ten object can only be attached once.")
	}

	p.attachToType = tenAttachToExtension
	p.attachTo = unsafe.Pointer(ext)
}

func (p *tenEnv) attachToApp(app *app) {
	if p.attachToType != tenAttachToInvalid {
		panic("The ten object can only be attached once.")
	}

	p.attachToType = tenAttachToApp
	p.attachTo = unsafe.Pointer(app)
}

func (p *tenEnv) postSyncJob(payload job) any {
	// To prevent deadlock, we refuse to post jobs to the pool. So it's
	// recommended for developers to call async ten apis in goroutines other
	// than the ten goroutine.
	return payload()
}

func (p *tenEnv) postAsyncJob(payload job) any {
	return p.process(payload)
}

func (p *tenEnv) SendCmd(cmd Cmd, handler ResultHandler) error {
	if cmd == nil {
		return newTenError(
			ErrnoInvalidArgument,
			"cmd is required.",
		)
	}

	return withCGO(func() error {
		return p.sendCmd(cmd, handler)
	})
}

func (p *tenEnv) sendCmd(cmd Cmd, handler ResultHandler) error {
	defer cmd.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_send_cmd(
		p.cPtr,
		cmd.getCPtr(),
		cHandle(cb),
		C.bool(false),
	)

	return withCGoError(&cStatus)
}

func (p *tenEnv) SendCmdEx(cmd Cmd, handler ResultHandler) error {
	if cmd == nil {
		return newTenError(
			ErrnoInvalidArgument,
			"cmd is required.",
		)
	}

	return withCGO(func() error {
		return p.sendCmdEx(cmd, handler)
	})
}

func (p *tenEnv) sendCmdEx(cmd Cmd, handler ResultHandler) error {
	defer cmd.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	cStatus := C.ten_go_ten_env_send_cmd(
		p.cPtr,
		cmd.getCPtr(),
		cHandle(cb),
		C.bool(true),
	)

	return withCGoError(&cStatus)
}

// Exported function to be called from C when the async operation in C
// completes.
//
//export tenGoCAsyncApiCallback
func tenGoCAsyncApiCallback(
	callbackHandle C.uintptr_t,
	apiStatus C.ten_go_error_t,
) {
	// Start a Go routine for asynchronous processing to prevent blocking C code
	// on the native thread, which would in turn block the Go code calling the C
	// code.
	go func() {
		goHandle := goHandle(callbackHandle)
		done := loadAndDeleteGoHandle(goHandle).(chan error)

		err := withCGoError(&apiStatus)

		done <- err
	}()
}

func (p *tenEnv) SendData(data Data, handler ErrorHandler) error {
	if data == nil {
		return newTenError(
			ErrnoInvalidArgument,
			"data is required.",
		)
	}

	defer data.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	err := withCGO(func() error {
		apiStatus := C.ten_go_ten_env_send_data(
			p.cPtr,
			data.getCPtr(),
			cHandle(cb),
		)
		err := withCGoError(&apiStatus)
		return err
	})

	if err != nil {
		// Clean up the handle if there was an error.
		loadAndDeleteGoHandle(cb)
	}

	return err
}

func (p *tenEnv) SendVideoFrame(
	videoFrame VideoFrame,
	handler ErrorHandler,
) error {
	if videoFrame == nil {
		return newTenError(
			ErrnoInvalidArgument,
			"videoFrame is required.",
		)
	}

	defer videoFrame.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	err := withCGO(func() error {
		apiStatus := C.ten_go_ten_env_send_video_frame(
			p.cPtr,
			videoFrame.getCPtr(),
			cHandle(cb),
		)
		return withCGoError(&apiStatus)
	})

	if err != nil {
		// Clean up the handle if there was an error.
		loadAndDeleteGoHandle(cb)
	}

	return err
}

func (p *tenEnv) SendAudioFrame(
	audioFrame AudioFrame,
	handler ErrorHandler,
) error {
	if audioFrame == nil {
		return newTenError(
			ErrnoInvalidArgument,
			"audioFrame is required.",
		)
	}

	defer audioFrame.keepAlive()

	cb := goHandleNil
	if handler != nil {
		cb = newGoHandle(handler)
	}

	err := withCGO(func() error {
		apiStatus := C.ten_go_ten_env_send_audio_frame(
			p.cPtr,
			audioFrame.getCPtr(),
			cHandle(cb),
		)
		return withCGoError(&apiStatus)
	})

	if err != nil {
		// Clean up the handle if there was an error.
		loadAndDeleteGoHandle(cb)
	}

	return err
}

func (p *tenEnv) OnConfigureDone() error {
	p.LogDebug("OnConfigureDone")

	if p.attachToType == tenAttachToApp {
		if err := RegisterAllAddons(nil); err != nil {
			p.LogFatal("Failed to register all GO addons: " + err.Error())
			return nil
		}
	}

	C.ten_go_ten_env_on_configure_done(p.cPtr)

	return nil
}

func (p *tenEnv) OnInitDone() error {
	p.LogDebug("OnInitDone")
	C.ten_go_ten_env_on_init_done(p.cPtr)

	return nil
}

func (p *tenEnv) OnStartDone() error {
	C.ten_go_ten_env_on_start_done(p.cPtr)

	return nil
}

func (p *tenEnv) OnStopDone() error {
	C.ten_go_ten_env_on_stop_done(p.cPtr)

	return nil
}

func (p *tenEnv) OnDeinitDone() error {
	C.ten_go_ten_env_on_deinit_done(p.cPtr)

	return nil
}

func (p *tenEnv) OnCreateInstanceDone(instance any, context uintptr) error {
	switch instance := instance.(type) {
	case *extension:
		C.ten_go_ten_env_on_create_instance_done(p.cPtr, instance.cPtr, C.uintptr_t(context))
	default:
		panic("instance must be extension or extension group.")
	}

	return nil
}

func (p *tenEnv) String() string {
	cString := C.ten_go_ten_env_debug_info(p.cPtr)
	defer C.free(unsafe.Pointer(cString))

	return C.GoString(cString)
}

func (p *tenEnv) LogVerbose(msg string) {
	p.logInternal(
		LogLevelVerbose, msg, 2)
}

func (p *tenEnv) LogDebug(msg string) {
	p.logInternal(LogLevelDebug, msg, 2)
}

func (p *tenEnv) LogInfo(msg string) {
	p.logInternal(LogLevelInfo, msg, 2)
}

func (p *tenEnv) LogWarn(msg string) {
	p.logInternal(LogLevelWarn, msg, 2)
}

func (p *tenEnv) LogError(msg string) {
	p.logInternal(LogLevelError, msg, 2)
}

func (p *tenEnv) LogFatal(msg string) {
	p.logInternal(LogLevelFatal, msg, 2)
}

func (p *tenEnv) logInternal(level LogLevel, msg string, skip int) {
	// Get caller info.
	pc, fileName, lineNo, ok := runtime.Caller(skip)
	funcName := "unknown"
	if ok {
		fn := runtime.FuncForPC(pc)
		if fn != nil {
			funcName = fn.Name()

			parts := strings.Split(funcName, ".")
			if len(parts) > 0 {
				// The last part is the method name.
				funcName = parts[len(parts)-1]
			}
		}
	} else {
		fileName = "unknown"
		lineNo = 0
	}

	C.ten_go_ten_env_log(
		p.cPtr,
		C.int(level),
		unsafe.Pointer(unsafe.StringData(funcName)),
		C.int(len(funcName)),
		unsafe.Pointer(unsafe.StringData(fileName)),
		C.int(len(fileName)),
		C.int(lineNo),
		unsafe.Pointer(unsafe.StringData(msg)),
		C.int(len(msg)),
	)
}
