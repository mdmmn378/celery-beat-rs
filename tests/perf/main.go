package main

import (
	"encoding/json"
	"net/http"
)

type HealthResponse struct {
	Status string `json:"status"`
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	response := HealthResponse{Status: "ok"}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func main() {
	http.HandleFunc("/api/v1/health/", healthHandler)

	// Start the server on port 8080
	if err := http.ListenAndServe(":8080", nil); err != nil {
		panic(err)
	}
}
