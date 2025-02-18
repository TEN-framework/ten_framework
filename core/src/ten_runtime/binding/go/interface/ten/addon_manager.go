//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package ten

// #include "addon.h"
import "C"

import (
	"fmt"
	"runtime"
	"sync"
	"unsafe"
)

// AddonManager is a manager for addons.
type AddonManager struct {
	// Define a registry map to store addon registration functions.
	// The key is the addonName (string), and the value is a function that takes
	// a registerCtx (interface{}) and returns an error.
	registry map[string]func(interface{}) error

	registryMutex sync.RWMutex
}

func newAddonManager() *AddonManager {
	return &AddonManager{
		registry: make(map[string]func(interface{}) error),
	}
}

// RegisterAddonAsExtension registers the addon as an extension.
func (am *AddonManager) RegisterAddonAsExtension(
	addonName string,
	instance Addon,
) error {
	if len(addonName) == 0 {
		return newTenError(
			ErrorCodeInvalidArgument,
			"addon name is empty",
		)
	}

	// Define the registration function that will be stored in the registry.
	registerHandler := func(registerCtx interface{}) error {
		addonWrapper := &addon{
			Addon: instance,
		}

		addonID := newImmutableHandle(addonWrapper)

		var bridge C.uintptr_t
		cgoError := C.ten_go_addon_register_extension(
			unsafe.Pointer(unsafe.StringData(addonName)),
			C.int(len(addonName)),
			cHandle(addonID),
			// In the current use of the TEN framework's GO environment, there
			// is no need to pass any `register_ctx` object into the register
			// handler of the GO addon. Therefore, for now, simply passing `nil`
			// is sufficient. If needed in the future, we can consider what
			// information should be passed to the register handler of the GO
			// addon.
			nil,
			&bridge,
		)

		if err := withCGoError(&cgoError); err != nil {
			loadAndDeleteImmutableHandle(addonID)
			return err
		}

		addonWrapper.cPtr = bridge

		runtime.SetFinalizer(addonWrapper, func(p *addon) {
			C.ten_go_addon_unregister(p.cPtr)
		})

		return nil
	}

	// Store the registration function in the registry map.
	am.registryMutex.Lock()
	defer am.registryMutex.Unlock()

	if _, exists := am.registry[addonName]; exists {
		return newTenError(
			ErrorCodeInvalidArgument,
			fmt.Sprintf("Addon '%s' is already registered", addonName),
		)
	}

	am.registry[addonName] = registerHandler

	return nil
}

// RegisterAllAddons executes all registered addon registration functions.
func (am *AddonManager) RegisterAllAddons(registerCtx interface{}) error {
	am.registryMutex.Lock()
	defer am.registryMutex.Unlock()

	for name, registerHandler := range am.registry {
		if err := registerHandler(registerCtx); err != nil {
			return fmt.Errorf("Failed to register addon %s: %w", name, err)
		}
	}

	// Clear the registry to free up memory.
	am.registry = make(map[string]func(interface{}) error)

	return nil
}

var defaultAddonManager = newAddonManager()

// RegisterAddonAsExtension registers the addon as an extension.
func RegisterAddonAsExtension(addonName string, instance Addon) error {
	return defaultAddonManager.RegisterAddonAsExtension(addonName, instance)
}

// RegisterAllAddons executes all registered addon registration functions.
func RegisterAllAddons(registerCtx interface{}) error {
	return defaultAddonManager.RegisterAllAddons(registerCtx)
}
