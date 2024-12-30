//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package ten

// #include "extension.h"
import "C"

import (
	"fmt"
	"log"
	"runtime"
	"unsafe"
)

type Extension interface {
	OnConfigure(tenEnv TenEnv)
	OnInit(
		tenEnv TenEnv,
	)
	OnStart(tenEnv TenEnv)
	OnStop(tenEnv TenEnv)
	OnDeinit(tenEnv TenEnv)
	OnCmd(tenEnv TenEnv, cmd Cmd)
	OnData(tenEnv TenEnv, data Data)
	OnVideoFrame(tenEnv TenEnv, videoFrame VideoFrame)
	OnAudioFrame(tenEnv TenEnv, audioFrame AudioFrame)
}

// DefaultExtension implements the Extension interface.
type DefaultExtension struct{}

var _ Extension = new(DefaultExtension)

func (p *DefaultExtension) OnConfigure(tenEnv TenEnv) {
	tenEnv.OnConfigureDone()
}

func (p *DefaultExtension) OnInit(
	tenEnv TenEnv,
) {
	tenEnv.OnInitDone()
}

func (p *DefaultExtension) OnStart(tenEnv TenEnv) {
	tenEnv.OnStartDone()
}

func (p *DefaultExtension) OnStop(tenEnv TenEnv) {
	tenEnv.OnStopDone()
}

func (p *DefaultExtension) OnDeinit(tenEnv TenEnv) {
	tenEnv.OnDeinitDone()
}

func (p *DefaultExtension) OnCmd(tenEnv TenEnv, cmd Cmd) {
}

func (p *DefaultExtension) OnData(tenEnv TenEnv, data Data) {
}

func (p *DefaultExtension) OnVideoFrame(
	tenEnv TenEnv,
	videoFrame VideoFrame,
) {
}

func (p *DefaultExtension) OnAudioFrame(
	tenEnv TenEnv,
	audioFrame AudioFrame,
) {
}

// The GO Extension object created by the GO binding (refer to WrapExtension),
// which is bound to the C Extension. The first field of `extension` is the
// user-defined extension instance (i.e., Extension).
//
// Via embedding, the `extension` struct "inherits" all methods of Extension.
type extension struct {
	Extension
	baseTenObject[C.uintptr_t]
}

func WrapExtension(
	ext Extension,
	name string,
) Extension {
	extInstance := &extension{
		Extension: ext,
	}

	extObjID := newImmutableHandle(extInstance)

	var bridge C.uintptr_t
	cgo_error := C.ten_go_extension_create(
		cHandle(extObjID),
		unsafe.Pointer(unsafe.StringData(name)),
		C.int(len(name)),
		&bridge,
	)
	if err := withCGoError(&cgo_error); err != nil {
		log.Printf("Failed to create extension, %v\n", err)
		return nil
	}

	extInstance.cPtr = bridge

	runtime.SetFinalizer(extInstance, func(p *extension) {
		C.ten_go_extension_finalize(p.cPtr)
	})

	return extInstance
}

func newExtensionWithBridge(
	ext Extension,
	_name string,
	bridge C.uintptr_t,
) goHandle {
	instance := &extension{
		Extension: ext,
	}

	instance.cPtr = bridge

	runtime.SetFinalizer(instance, func(p *extension) {
		C.ten_go_extension_finalize(p.cPtr)
	})

	return newImmutableHandle(instance)
}

//export tenGoExtensionOnConfigure
func tenGoExtensionOnConfigure(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	tenEnvInstance, ok := tenEnvObj.(*tenEnv)
	if !ok {
		// Should not happen.
		panic("Invalid ten object type.")
	}

	tenEnvInstance.attachToExtension(extensionObj)

	extensionObj.OnConfigure(tenEnvObj)
}

//export tenGoExtensionOnInit
func tenGoExtensionOnInit(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	tenEnvInstance, ok := tenEnvObj.(*tenEnv)
	if tenEnvInstance == nil || !ok {
		// Should not happen.
		panic("Invalid ten object type.")
	}

	// As the `extension` struct embeds a user-defined extension instance
	// implements the Extension interface, the `OnInit` method can be called
	// directly on the `extensionObj` object. The receiver of the `OnInit`
	// method is actually the user-defined extension instance. The calling
	// conversation is same as the following:
	//
	//   extensionObj.Extension.OnInit(tenEnvObj, propertyObj)
	//
	extensionObj.OnInit(tenEnvObj)
}

//export tenGoExtensionOnStart
func tenGoExtensionOnStart(extensionID C.uintptr_t, tenEnvID C.uintptr_t) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	extensionObj.OnStart(tenEnvObj)
}

//export tenGoExtensionOnStop
func tenGoExtensionOnStop(extensionID C.uintptr_t, tenEnvID C.uintptr_t) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	extensionObj.OnStop(tenEnvObj)
}

//export tenGoExtensionOnDeinit
func tenGoExtensionOnDeinit(extensionID C.uintptr_t, tenEnvID C.uintptr_t) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	extensionObj.OnDeinit(tenEnvObj)
}

//export tenGoExtensionOnCmd
func tenGoExtensionOnCmd(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
	cmdBridge C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	// The GO cmd object should be created in GO side, and managed by the GO GC.
	customCmd := newCmd(cmdBridge)
	extensionObj.OnCmd(tenEnvObj, customCmd)
}

//export tenGoExtensionOnData
func tenGoExtensionOnData(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
	dataBridge C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	// The GO data object should be created in GO side, and managed by the GO
	// GC.
	d := newData(dataBridge)
	extensionObj.OnData(tenEnvObj, d)
}

//export tenGoExtensionOnVideoFrame
func tenGoExtensionOnVideoFrame(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
	videoFrameBridge C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	videoFrameObj := newVideoFrame(videoFrameBridge)
	extensionObj.OnVideoFrame(tenEnvObj, videoFrameObj)
}

//export tenGoExtensionOnAudioFrame
func tenGoExtensionOnAudioFrame(
	extensionID C.uintptr_t,
	tenEnvID C.uintptr_t,
	audioFrameBridge C.uintptr_t,
) {
	extensionObj, ok := loadImmutableHandle(goHandle(extensionID)).(*extension)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get extension from handle map, id: %d.",
				uintptr(extensionID),
			),
		)
	}

	tenEnvObj, ok := handle(tenEnvID).get().(TenEnv)
	if !ok {
		panic(
			fmt.Sprintf(
				"Failed to get ten env from handle map, id: %d.",
				uintptr(tenEnvID),
			),
		)
	}

	audioFrameObj := newAudioFrame(audioFrameBridge)
	extensionObj.OnAudioFrame(tenEnvObj, audioFrameObj)
}
