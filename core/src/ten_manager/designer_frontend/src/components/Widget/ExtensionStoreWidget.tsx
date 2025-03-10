//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
// import * as React from "react";

import { useListTenCloudStorePackages } from "@/api/services/extension";
import { SpinnerLoading } from "@/components/Status/Loading";

export const ExtensionStoreWidget = () => {
  const { data, error, isLoading } = useListTenCloudStorePackages();
  return (
    <div>
      {isLoading && <SpinnerLoading />}
      {error && <div>Error: {error.message}</div>}
      {data && <div>Data: {JSON.stringify(data)}</div>}
    </div>
  );
};
