/*******************************************************************************
*   Ledger Blue
*   (c) 2016 Ledger
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

#include "os.h"

typedef struct tokenDefinition_t {
    uint8_t address[20];
    uint8_t ticker[10];
    uint8_t decimals;
} tokenDefinition_t;

#define NUM_TOKENS_AKROMA 0
#define NUM_TOKENS_ETHEREUM 677
#define NUM_TOKENS_ETHEREUM_CLASSIC 0
#define NUM_TOKENS_POA 0
#define NUM_TOKENS_RSK 0
#define NUM_TOKENS_UBIQ 6
#define NUM_TOKENS_EXPANSE 0
#define NUM_TOKENS_WANCHAIN 0
#define NUM_TOKENS_KUSD 0

extern tokenDefinition_t const TOKENS_AKROMA[NUM_TOKENS_AKROMA];
extern tokenDefinition_t const TOKENS_ETHEREUM[NUM_TOKENS_ETHEREUM];
extern tokenDefinition_t const TOKENS_ETHEREUM_CLASSIC[NUM_TOKENS_ETHEREUM_CLASSIC];
extern tokenDefinition_t const TOKENS_POA[NUM_TOKENS_POA];
extern tokenDefinition_t const TOKENS_RSK[NUM_TOKENS_RSK];
extern tokenDefinition_t const TOKENS_UBIQ[NUM_TOKENS_UBIQ];
extern tokenDefinition_t const TOKENS_EXPANSE[NUM_TOKENS_EXPANSE];
extern tokenDefinition_t const TOKENS_WANCHAIN[NUM_TOKENS_WANCHAIN];
extern tokenDefinition_t const TOKENS_KUSD[NUM_TOKENS_KUSD];
