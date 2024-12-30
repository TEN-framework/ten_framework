//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

package ten

import (
	"encoding/json"
	"math/rand"
	"runtime"
	"strconv"
	"testing"
	"time"
)

type payload struct {
	MsgId     string `json:"msg_id"`
	OpenId    string `json:"openid"`
	Content   string `json:"content"`
	AvatarUrl string `json:"avatar_url"`
	Nickname  string `json:"nickname"`
	Timestamp int64  `json:"timestamp"`
}

type liveMessage struct {
	RoomId   string    `json:"room_id"`
	AppId    string    `json:"app_id"`
	Payloads []payload `json:"payloads"`
}

// N = 1, size = 267
// N = 2, size = 455
// N = 10, size = 1974
// N = 20, size = 3861
func TestBytesSize(t *testing.T) {
	msg := &liveMessage{
		RoomId: strconv.FormatInt(rand.Int63(), 10),
		AppId:  strconv.FormatInt(rand.Int63(), 10),
	}

	N := 20

	msg.Payloads = make([]payload, 0, N)
	for i := 0; i < N; i++ {
		msg.Payloads = append(msg.Payloads, payload{
			MsgId:     strconv.FormatInt(rand.Int63(), 10),
			OpenId:    strconv.FormatInt(rand.Int63(), 10),
			Content:   strconv.FormatInt(rand.Int63(), 10),
			AvatarUrl: strconv.FormatInt(rand.Int63(), 10),
			Nickname:  strconv.FormatInt(rand.Int63(), 10),
			Timestamp: time.Now().UnixMilli(),
		})
	}

	b, _ := json.Marshal(msg)
	t.Logf("msg size: %d", len(b))
}

// The benchmark is:
//
// goos: linux
// goarch: amd64
// cpu: Intel(R) Core(TM) i7-10510U CPU @ 1.80GHz
//
// -8   	15522892	        67.27 ns/op	      24 B/op	       1 allocs/op
func BenchmarkBytesPool(b *testing.B) {
	r := rand.New(rand.NewSource(time.Now().UnixNano()))

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		size := r.Intn(defaultBoundaryHuge)

		buf := acquireBytes(size)
		if buf == nil || cap(buf) < size {
			b.FailNow()
		}

		buf = buf[:size]

		ReleaseBytes(buf)
	}

	b.ReportAllocs()
}

// The benchmark is:
// goos: linux
// goarch: amd64
// cpu: Intel(R) Core(TM) i7-10510U CPU @ 1.80GHz
//
// -8   	40925986	        24.48 ns/op	      24 B/op	       1 allocs/op
func BenchmarkBytesPoolWithGC(b *testing.B) {
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		r := rand.New(rand.NewSource(time.Now().UnixNano()))
		for pb.Next() {
			size := r.Intn(defaultBoundaryHuge)

			buf := acquireBytes(size)
			if buf == nil || cap(buf) < size {
				b.FailNow()
			}

			ReleaseBytes(buf)
		}

		runtime.GC()
	})

	b.ReportAllocs()
}
