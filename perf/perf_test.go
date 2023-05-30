package main_test

import (
	"net/http"
	"testing"
)

func bmASGIConcurrency(b *testing.B, tgt string) {
	b.ResetTimer()
	b.RunParallel(func(p *testing.PB) {
		for p.Next() {
			_, err := http.Get(tgt)
			if err != nil {
				panic(err)
			}
		}
	})
}

// func BenchmarkUvicorn(b *testing.B) {
// 	bmASGIConcurrency(b, "http://localhost:9009")
// }

func BenchmarkHypercorn(b *testing.B) {
	bmASGIConcurrency(b, "http://localhost:9010")
}

// func BenchmarkBootstrap(b *testing.B) {
// 	bmASGIConcurrency(b, "http://localhost:8000")
// }
