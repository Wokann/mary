/* FoMT ordered callable IDs. Select exactly one of MARY_FOMT_JP,
 * MARY_FOMT_US, MARY_FOMT_EU, or MARY_FOMT_DE.
 *
 * Evidence provenance: the current `fomt` source tree represents FoMT-US.
 * Verified localization differences are selected by the matching REGION_*
 * macro derived from the selected target.
 *
 * FoMT 有序 callable ID。必须且只能选择 MARY_FOMT_JP、MARY_FOMT_US、
 * MARY_FOMT_EU 或 MARY_FOMT_DE 之一。
 *
 * 证据来源说明：当前 `fomt` 源码树代表 FoMT-US；经过验证的本地化差异由
 * 所选目标派生出的对应 REGION_* 宏选择。 */
mary_callable_table
{
    /* VM-internal stack/control slots, not source-level calls. */
    /* 0x000 */ NULL,
    /* 0x001 */ NULL,
    /* 0x002 */ SetEntityPosition,
    /* 0x003 */ GetEntityX,
    /* 0x004 */ GetEntityY,
    /* 0x005 */ SetEntityFacing,
    /* 0x006 */ GetEntityFacing,
    /* 0x007 */ SetEntitySpritePriority,
    /* 0x008 */ MoveEntityXTo,
    /* 0x009 */ MoveEntityXToRaw,
    /* 0x00A */ MoveEntityYTo,
    /* 0x00B */ MoveEntityYToRaw,
    /* 0x00C */ WaitForEntityMovement,
    /* 0x00D */ SetEntityAnim,
    /* 0x00E */ NoOp014,
    /* 0x00F */ HideEntity,
    /* 0x010 */ SetEntityAuxRenderProfile,
    /* 0x011 */ StartEntityEffect,
    /* 0x012 */ StopEntityEffect,
    /* 0x013 */ GetOppositeFacing,
    /* 0x014 */ GetEntityLocation,
    /* 0x015 */ OffsetEntityPosition,
    /* 0x016 */ ChangeMap,
    /* 0x017 */ PanCameraTo,
    /* 0x018 */ WaitForCameraMovement,
    /* 0x019 */ PlayBGM,
    /* 0x01A */ StopBGM,
    /* 0x01B */ PlaySong,
    /* 0x01C */ StopAllSongs,
    /* 0x01D */ FadeOutBGM,
    /* 0x01E */ ResetTalkUi,
    /* 0x01F */ TalkOpen,
    /* 0x020 */ TalkClose,
    /* 0x021 */ TalkMessage,
    /* 0x022 */ TalkMessageSlow,
    /* 0x023 */ TalkAppendMessage,
    /* 0x024 */ TalkPromptChoice2,
    /* 0x025 */ TalkPromptChoice3,
    /* 0x026 */ TalkPromptChoice4,
    /* 0x027 */ TalkChoice2,
    /* 0x028 */ TalkChoice3,
    /* 0x029 */ TalkChoice4,
    /* 0x02A */ TalkChoice5,
    /* 0x02B */ TalkChoice6,
    /* 0x02C */ SetTalkNameplateCharacter,
    /* 0x02D */ SetTalkNameplateText,
    /* 0x02E */ ClearTalkNameplate,
    /* 0x02F */ SetTalkPortrait,
    /* 0x030 */ ClearTalkPortrait,
    /* 0x031 */ ShowTalkHeartIndicator,
    /* 0x032 */ HideTalkHeartIndicator,
    /* 0x033 */ FadeOutScreen,
    /* 0x034 */ FadeInScreen,
    /* 0x035 */ FadeInScreenWithoutSceneHook,
    /* 0x036 */ WaitFrames,
    /* 0x037 */ CallScript,
    /* 0x038 */ SetTextVariableNumber,
    /* 0x039 */ SetTextVariableNumberFieldWidth,
    /* 0x03A */ SetTextVariableString,
    /* 0x03B */ RandomU15,
    /* 0x03C */ RandomIntInclusive,
    /* 0x03D */ VarGet,
    /* 0x03E */ VarSet,
    /* 0x03F */ IsPlayerHoldingNothing,
    /* 0x040 */ GetPlayerHeldItemKind,
    /* 0x041 */ IsPlayerHeldItemWrapped,
    /* 0x042 */ GetPlayerHeldFoodId,
    /* 0x043 */ GetPlayerHeldArticleId,
    /* 0x044 */ GetPlayerHeldChickenId,
    /* 0x045 */ UsePlayerHeldItem,
    /* 0x046 */ ClearPlayerHeldItem,
    /* 0x047 */ SetPlayerHeldFood,
    /* 0x048 */ SetPlayerHeldArticle,
    /* 0x049 */ SetPlayerHeldWrappedFood,
    /* 0x04A */ SetPlayerHeldWrappedArticle,
    /* 0x04B */ CanDiscardPlayerHeldArticle,
    /* 0x04C */ TryShipPlayerHeldItem,
    /* 0x04D */ ThrowPlayerHeldItem,
    /* 0x04E */ GetPlayerHeldToolId,
    /* 0x04F */ GetPlayerHeldToolStackCount,
    /* 0x050 */ SetPlayerHeldTool,
    /* 0x051 */ ClearPlayerHeldTool,
    /* 0x052 */ FindFoodInRucksack,
    /* 0x053 */ FindArticleInRucksack,
    /* 0x054 */ ClearRucksackItemSlot,
    /* 0x055 */ GetFirstFreeRucksackToolSlot,
    /* 0x056 */ GetFirstFreeRucksackItemSlot,
    /* 0x057 */ AddArticleToRucksack,
    /* 0x058 */ AddFoodToRucksack,
    /* 0x059 */ AddToolToRucksack,
    /* 0x05A */ ShowPlayerHoldingTool,
    /* 0x05B */ ChangePlayerStaminaAndFatigue,
    /* 0x05C */ IsPlayerHoldingTool,
    /* 0x05D */ PlayerOwnsTool,
    /* 0x05E */ PlayerOwnsFood,
    /* 0x05F */ PlayerOwnsArticle,
    /* 0x060 */ RemoveAllOwnedArticles,
    /* 0x061 */ ObtainPowerBerry,
    /* 0x062 */ ObtainMysticBerry,
    /* 0x063 */ IsPlayerRidingHorse,
    /* 0x064 */ EnterHotSpringBathingState,
    /* 0x065 */ ExitHotSpringBathingState,
    /* 0x066 */ PreservePlayerLocationForNextDay,
    /* 0x067 */ ClearPreservedPlayerLocation,
    /* 0x068 */ GetPreservedPlayerMapId,
    /* 0x069 */ EatRandomMeal,
    /* 0x06A */ PreparePlayerForScriptedAnimation,
    /* 0x06B */ RestorePlayerAfterScriptedAnimation,
    /* 0x06C */ GetPresentedItemKind,
    /* 0x06D */ GetPresentedItemId,
    /* 0x06E */ IsPresentedItemGiftWrapped,
    /* 0x06F */ CanReceiveTool,
    /* 0x070 */ CanReceiveFood,
    /* 0x071 */ CanReceiveArticle,
    /* 0x072 */ GivePlayerBasket,
    /* 0x073 */ PlayerHasBasket,
    /* 0x074 */ UpgradeRucksack,
    /* 0x075 */ GetRucksackUpgradeLevel,
    /* 0x076 */ SetPlayerActorUpdateSuspended,
    /* 0x077 */ HasLocalLinkMilestone,
    /* 0x078 */ HasReceivedLinkMilestone,
    /* 0x079 */ SetLocalLinkMilestone,
    /* 0x07A */ ClearLocalLinkMilestone,
    /* 0x07B */ IsCharacterBirthdayToday,
    /* 0x07C */ GetNpcFriendship,
    /* 0x07D */ AddNpcFriendship,
    /* 0x07E */ SetNpcFriendship,
    /* 0x07F */ GetDaysSinceLastSpokenToNpc,
    /* 0x080 */ MarkNpcSpokenTo,
    /* 0x081 */ WasNpcSpokenToToday,
    /* 0x082 */ WasNpcSpokenToJustNow,
    /* 0x083 */ HasMetNpc,
    /* 0x084 */ MarkNpcGifted,
    /* 0x085 */ WasNpcGiftedToday,
    /* 0x086 */ GetCharacterLove,
    /* 0x087 */ AddCharacterLove,
    /* 0x088 */ SetCharacterLove,
    /* 0x089 */ SetEntityEventScript,
    /* 0x08A */ ClearEntityEventScript,
    /* 0x08B */ OpenSupermarketShop,
    /* 0x08C */ PurchaseSupermarketItem,
    /* 0x08D */ OpenWonShop,
    /* 0x08E */ OpenCarpenterShop,
    /* 0x08F */ OpenBlacksmithShop,
    /* 0x090 */ OpenClinicShop,
    /* 0x091 */ OpenBeachCafeShop,
    /* 0x092 */ OpenYodelRanchShop,
    /* 0x093 */ OpenWineryShop,
    /* 0x094 */ OpenInnShop,
    /* 0x095 */ OpenPoultryFarmShop,
    /* 0x096 */ OpenSpecialMerchantShop,
    /* 0x097 */ OpenGiftWrappingMenu,
    /* 0x098 */ ShowReferencePage,
    /* 0x099 */ OpenBookList,
    /* 0x09A */ OpenLetterList,
    /* 0x09B */ OpenCalendar,
    /* 0x09C */ OpenShelf,
    /* 0x09D */ OpenToolChest,
    /* 0x09E */ OpenRefrigerator,
    /* 0x09F */ OpenClock,
    /* 0x0A0 */ OpenCookingMenu,
    /* 0x0A1 */ OpenRecipeList,
    /* 0x0A2 */ RunGameCubeLink,
    /* 0x0A3 */ OpenNameEntry,
    /* 0x0A4 */ StartFarmInheritanceFlashback,
    /* 0x0A5 */ OpenNameEntryKeyboard,
    /* 0x0A6 */ OpenRucksackMenu,
    /* 0x0A7 */ SelectFestivalAnimal,
    /* 0x0A8 */ RunStaffCredits,
    /* 0x0A9 */ OpenFarmingTutorial,
    /* 0x0AA */ PrepareClockMenuTransition,
    /* 0x0AB */ RestoreAfterClockMenu,
    /* 0x0AC */ PrepareCookingMenuTransition,
    /* 0x0AD */ RestoreAfterCookingMenu,
    /* 0x0AE */ PrepareRecipeMenuTransition,
    /* 0x0AF */ RestoreAfterRecipeMenu,
    /* 0x0B0 */ RecordPlayerHasAlbum,
    /* 0x0B1 */ SwapRecordPlayerAlbum,
    /* 0x0B2 */ RemoveRecordPlayerAlbum,
    /* 0x0B3 */ LightFireplaceAtLocation,
    /* 0x0B4 */ IsFireplaceLitAtLocation,
    /* 0x0B5 */ SetVaseArticleId,
    /* 0x0B6 */ GetVaseArticleId,
    /* 0x0B7 */ IsChickenFeedTroughFilled,
    /* 0x0B8 */ FillChickenFeedTrough,
    /* 0x0B9 */ BeginEggIncubation,
    /* 0x0BA */ IsIncubatorOccupied,
    /* 0x0BB */ GetIncubatorCapacity,
    /* 0x0BC */ IsEggReadyToHatch,
    /* 0x0BD */ AttemptEggHatch,
    /* 0x0BE */ IsBarnFeedTroughFilled,
    /* 0x0BF */ FillBarnFeedTrough,
    /* 0x0C0 */ StartShipmentBoxDepositAnimation,
    /* 0x0C1 */ GetPregnancyStallCapacity,
    /* 0x0C2 */ IsBarnAnimalReadyToGiveBirth,
    /* 0x0C3 */ AttemptBarnAnimalBirth,
    /* 0x0C4 */ BuildMountainCottage,
    /* 0x0C5 */ BuildSeasideCottage,
    /* 0x0C6 */ HasGoldenLumberOnFarm,
    /* 0x0C7 */ OpenDoor,
    /* 0x0C8 */ CloseDoor,
    /* 0x0C9 */ RedrawRucksackShelfAfterPurchase,
    /* 0x0CA */ RedrawBlueFeatherShelfAfterPurchase,
    /* 0x0CB */ GetHarvestSpriteCurrentTask,
    /* 0x0CC */ GetHarvestSpriteWorkDaysLeft,
    /* 0x0CD */ GetHarvestSpriteTaskExperience,
    /* 0x0CE */ HasHarvestSpritePlayedMinigameToday,
    /* 0x0CF */ HasHarvestSpriteMinigameExperience,
    /* 0x0D0 */ StartHarvestSpriteTask,
    /* 0x0D1 */ ScheduleHarvestSpriteTaskToEndAfterToday,
    /* 0x0D2 */ IsHarvestSpriteDailyWorkComplete,
    /* 0x0D3 */ RunHarvestSpriteAnimalCareMinigame,
    /* 0x0D4 */ RunHarvestSpriteHarvestingMinigame,
    /* 0x0D5 */ RunHarvestSpriteWateringMinigame,
    /* 0x0D6 */ RunChickenFestivalContest,
    /* 0x0D7 */ RunHorseRace,
    /* 0x0D8 */ PrepareHorseRaceEntries,
    /* 0x0D9 */ OpenHorseRaceMedalExchange,
    /* 0x0DA */ RunFrisbeeGame,
    /* 0x0DB */ RunFrisbeeTournamentRound,
    /* 0x0DC */ PrepareAnimalFestivalOpponents,
    /* 0x0DD */ GetTVShoppingSelection,
    /* 0x0DE */ GetPendingTVShoppingItem,
    /* 0x0DF */ IsTVShoppingDeliveryReady,
    /* 0x0E0 */ SetTVShoppingSelection,
    /* 0x0E1 */ ConfirmTVShoppingOrder,
    /* 0x0E2 */ CompleteTVShoppingDelivery,
    /* 0x0E3 */ IsVacationVillaBuilt,
    /* 0x0E4 */ AreAllRequiredVillagersAtMaxFriendship,
    /* 0x0E5 */ HasShippedOneOfEachCrop,
    /* 0x0E6 */ AreAllFarmAnimalsAtMaxAffection,
    /* 0x0E7 */ HasShippedOneOfEachMineral,
    /* 0x0E8 */ HasCaughtEveryFishSpecies,
    /* 0x0E9 */ GetTotalFishCaught,
    /* 0x0EA */ HasObtainedMythicTool,
    /* 0x0EB */ HasShippedOneOfEachProduct,
    /* 0x0EC */ GetMoney,
    /* 0x0ED */ AddMoney,
    /* 0x0EE */ SubtractMoney,
    /* 0x0EF */ EnableScriptedNpcControl,
    /* 0x0F0 */ DisableScriptedNpcControl,
    /* 0x0F1 */ GetBlacksmithOrderId,
    /* 0x0F2 */ IsBlacksmithOrderReady,
    /* 0x0F3 */ CollectBlacksmithOrder,
    /* 0x0F4 */ SetGameTime,
    /* 0x0F5 */ IsLetterWaiting,
    /* 0x0F6 */ HasReceivedLetter,
    /* 0x0F7 */ DeliverLetter,
    /* 0x0F8 */ MarkLetterRead,
    /* 0x0F9 */ GetWaitingLetterCount,
    /* 0x0FA */ GetSavedLetterCount,
    /* 0x0FB */ ShowTelevisionMessage,
    /* 0x0FC */ SetTelevisionProgram,
    /* 0x0FD */ EndTelevisionProgram,
    /* 0x0FE */ RefreshAllNpcSchedules,
    /* 0x0FF */ DoesAnimalExist,
    /* 0x100 */ CreateFarmHorse,
    /* 0x101 */ RemoveFarmHorse,
    /* 0x102 */ GetAnimalName,
    /* 0x103 */ GetInteractingAnimalIndex,
    /* 0x104 */ HasAnimalBeenTalkedTo,
    /* 0x105 */ SetAnimalTalkedTo,
    /* 0x106 */ AddAnimalAffection,
    /* 0x107 */ IsAnimalUnhappy,
    /* 0x108 */ IsAnimalSick,
    /* 0x109 */ IsAnimalPregnant,
    /* 0x10A */ GetAnimalHealthyPregnancyDays,
    /* 0x10B */ GetAnimalAge,
    /* 0x10C */ GetAnimalAffection,
    /* 0x10D */ GetAnimalGrowthStage,
    /* 0x10E */ IsSheepSheared,
    /* 0x10F */ CountAnimalsByLifeState,
    /* 0x110 */ ShowLivestockNeglectDeathSummary,
    /* 0x111 */ RemoveLivestockDeadFromNeglect,
    /* 0x112 */ ShowNaturalLivestockDeathSummary,
    /* 0x113 */ RemoveNaturallyDeadLivestock,
    /* 0x114 */ IsCowAtBarnSlot,
    /* 0x115 */ GetCowCount,
    /* 0x116 */ GetSheepCount,
    /* 0x117 */ GetChickenCount,
    /* 0x118 */ SetContestAnimal,
    /* 0x119 */ ClearContestAnimal,
    /* 0x11A */ GetContestAnimalIndex,
    /* 0x11B */ SetTextVariableToCaughtFishName,
    /* 0x11C */ GetCaughtFishSize,
    /* 0x11D */ IsCaughtFishMaximumSize,
    /* 0x11E */ IsCaughtFishKing,
    /* 0x11F */ SelectMoonViewingPartner,
    /* 0x120 */ GetCookingFestivalDishRating,
    /* 0x121 */ SelectThomasStockingGift,
    /* 0x122 */ GetRandomSpouseGiftArticleId,
    /* 0x123 */ UnlockNextVanAlbum,
    /* 0x124 */ AreAllVanAlbumsAvailable,
    /* 0x125 */ SetPlayerNicknameForSpouse,
    /* 0x126 */ PlacePlayerAtFarmhouseBed,
    /* 0x127 */ FlashScreenColor,
    /* 0x128 */ RebuildMapEntitiesForNewDay,
    /* 0x129 */ CureAllSickLivestock,
    /* 0x12A */ AddAffectionToAllFarmAnimals,
    /* 0x12B */ EnableShootingStarShippingBonus,
    /* 0x12C */ GetAmountShipped,
    /* 0x12D */ CreatePlayerChildEntity,
    /* 0x12E */ RelocateEntityToMap,
    /* 0x12F */ CreateNewYearSunriseEffect,
    /* 0x130 */ PlayNewYearSunriseEffect,
    /* 0x131 */ DestroyNewYearSunriseEffect,
    /* 0x132 */ PlayStarSparkleEffect,
    /* 0x133 */ GetKnownRecipeCount,
    /* 0x134 */ GenerateMineFloorLayout,
    /* 0x135 */ DescendMineFloor,
    /* 0x136 */ IsToolCursed,
    /* 0x137 */ AdvanceCursedToolLiftProgress,
    /* 0x138 */ AttemptChurchCursedToolRemoval,
    /* 0x139 */ CreateEventIcon,
    /* 0x13A */ RemoveEventIcon,
    /* 0x13B */ GetFoodIconId,
    /* 0x13C */ GetArticleIconId,
    /* 0x13D */ GetToolIconId,
    /* 0x13E */ NoOpTutorialFieldTile,
    /* 0x13F */ NoOpTutorialFieldObject,
    /* 0x140 */ NoOpTutorialEggDefinition,
    /* 0x141 */ NoOpTutorialEggSelection,
    /* 0x142 */ BeginHoldingActorGraphic,
    /* 0x143 */ NoOpAnimalEventEntityInitialization,
    /* 0x144 */ SetTextVariableToProductName,
    /* 0x145 */ GetEventContextValue,
    /* 0x146 */ CycleBackwardToNonCursedTool,
};

/*
 * Mary-C callable declarations
 *
 * Declaration order exactly follows mary_callable_table and therefore mirrors
 * its logical callable ID order.
 *
 * Mary*Id and Mary*Kind accept symbols from fomt_constants.mary.h or original
 * integers; both compile to identical bytes.
 *
 * Every callable currently exposed by name has been assigned from source and
 * native handler behavior, with script call sites where the shipped scripts
 * provide them. Remaining NULL slots are documented internal, duplicate, or
 * unreferenced entries rather than guessed declarations.
 *
 * Mary-C 可调用函数声明
 *
 * 声明顺序严格跟随 mary_callable_table，亦即逻辑 callable ID 顺序。
 *
 * Mary*Id、Mary*Kind 可使用 fomt_constants.mary.h 的符号或原始整数，
 * 两者编译字节完全相同。
 *
 * 当前所有具名 callable 均已根据源码和原生处理函数行为确定；原版脚本存在
 * 调用点时也会一并验证。剩余 NULL 均已标记为内部、重复或原版未引用入口，
 * 不会用猜测性声明填充。
 */

/*
 * Callable naming convention
 *
 * Semantic names follow the verb-first style used by established GBA
 * decompilation projects: Get/Set for stored values, Is/Has/Can for predicates,
 * Add/Remove/Clear for state changes, Show/Hide/Open/Close for presentation,
 * Play/Stop for audio, Start/Wait/Finish for multi-step operations, and
 * Create/Destroy for runtime objects. Related callables use the same domain
 * noun and differ only by the operation, so names remain searchable as a
 * family. The current public table contains no FuncXXX/ProcXXX placeholder;
 * where a business identity is not proven, the name states only the verified
 * physical or no-op behavior instead of inventing a semantic contract.
 *
 * callable 命名约定
 *
 * 语义名称采用成熟 GBA 反编译工程常见的“动词在前”风格：存储值使用
 * Get/Set，条件判断使用 Is/Has/Can，状态修改使用 Add/Remove/Clear，界面表现
 * 使用 Show/Hide/Open/Close，音频使用 Play/Stop，多阶段操作使用
 * Start/Wait/Finish，运行时对象使用 Create/Destroy。同一功能域的关联函数
 * 共用相同名词，仅以操作动词区分，以便按功能族检索。当前公开表已不存在
 * FuncXXX/ProcXXX 机械占位；业务身份尚未证实时，名称只陈述已确认的物理行为或
 * 空操作属性，不编造语义契约。
 */

/*
 * Audited legacy event helpers
 *
 * - Five late tutorial slots are verified retail no-ops. All four ROMs route
 *   them directly to the dispatcher return block without popping their
 *   operands or writing game state. Their declarations retain the observed
 *   operand counts solely so original bytecode can round-trip losslessly.
 * - MFoMT IsMapRegistered is a native callable slot absent from the older
 *   library. It tests membership in the initialized map-metadata registry; it
 *   does not test story access or whether the map is unlocked.
 *
 * 已审计的旧式事件辅助函数
 *
 * - 五个靠后的教程槽已经确认是零售版空操作。四个 ROM 都把它们直接指向
 *   dispatcher 返回块，不弹出操作数，也不写入游戏状态。声明中保留已观察到的
 *   操作数数量，仅用于让原始字节码无损往返。
 * - MFoMT IsMapRegistered 是旧库遗漏的原生 callable 槽，用于检测地图是否存在于
 *   初始化后的地图元数据注册表；它不表示剧情上可进入或已经解锁该地图。
 */

/*
 * Sets a runtime map entity's position and facing.
 * Parameters: entity_id is the runtime map entity ID; x and y are map
 * coordinates; facing is the engine direction value.
 * Uses the current scene's map, so this is not merely an X/Y assignment.
 * Positioning ENTITY_PLAYER on a mine floor also updates the shared deepest
 * floor reached record, only when the new floor is deeper. Both mines feed
 * that record; their map-number ranges differ between FoMT and MFoMT.
 * Player placement also enters family-specific visit-counter paths: FoMT's
 * library second floor feeds Mary's affection bonuses; MFoMT's church feeds
 * Cliff's. The MFoMT church path suppresses counting while the active romance
 * marker is Rick/Kai/Cliff/Gray/Doctor's marriage event, any of Cliff's four
 * heart events or proposal, or Kappa/Won/Gourmet's wedding event; it also
 * suppresses counting during the initial Music Festival phase. These are
 * guarded counters, not affection added on every call. Their repeat guards
 * are cleared by the daily state-update path reached from
 * RebuildMapEntitiesForNewDay in all four targets. Do not treat this call as
 * a side-effect-free visual operation.
 *
 * 设置运行时地图实体的位置与朝向。
 * 参数：entity_id 为运行时地图实体 ID；x、y 为地图坐标；facing 为引擎朝向值。
 * 使用当前场景的地图，因此不只是给 X/Y 赋值。将 ENTITY_PLAYER 定位到矿内
 * 楼层时，还会在楼层更深的情况下更新两座矿共用的历史最深到达层数记录。
 * FoMT 与 MFoMT 的矿地图编号区间不同。玩家定位还会进入版本特有的访问计数
 * 路径：FoMT 图书馆二楼的计数关联玛丽爱情度奖励，MFoMT 教堂的计数关联
 * 克里夫爱情度奖励。MFoMT 在当前活动恋爱事件标记属于 Rick／Kai／Cliff／
 * Gray／Doctor 的结婚事件、Cliff 的四阶段爱情事件或求婚事件、Kappa／Won／
 * Gourmet 的婚礼事件时不会计数；音乐节处于初始阶段时也不会计数。计数受
 * 重复保护，并非每次调用都增加爱情度；四个目标都会在
 * RebuildMapEntitiesForNewDay 进入的每日状态更新路径中清除该保护。不能把本
 * 函数当作没有其他副作用的纯显示操作。
 */
void SetEntityPosition(MaryEntityId entity_id, MaryMapSpaceX x, MaryMapSpaceY y, MaryFacingDirection facing);

/*
 * Returns a runtime map entity's X coordinate.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the signed integer part of the entity's Q16.16 map X
 * coordinate, or 0 when the entity is absent. Zero is also a valid coordinate.
 *
 * 返回运行时地图实体的 X 坐标。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体 Q16.16 地图 X 坐标的有符号整数部分；实体不存在时返回 0。
 * 0 本身也是合法坐标，不能用于判断实体是否存在。
 */
MaryMapSpaceX GetEntityX(MaryEntityId entity_id);

/*
 * Returns a runtime map entity's Y coordinate.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the signed integer part of the entity's Q16.16 map Y
 * coordinate, or 0 when the entity is absent. Zero is also a valid coordinate.
 *
 * 返回运行时地图实体的 Y 坐标。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体 Q16.16 地图 Y 坐标的有符号整数部分；实体不存在时返回 0。
 * 0 本身也是合法坐标，不能用于判断实体是否存在。
 */
MaryMapSpaceY GetEntityY(MaryEntityId entity_id);

/*
 * Sets a runtime map entity's facing direction.
 * Parameters: entity_id is the runtime map entity ID; facing is the engine
 * direction value. An absent entity or an unchanged direction is a no-op.
 * Changing direction refreshes the current animation base + facing; it does
 * not replace the animation group. The native setter stores a byte, not a
 * validated two-bit enum, so use FACING_DOWN/UP/LEFT/RIGHT rather than relying
 * on out-of-range values being rejected. All four SetAnimFacing bodies agree.
 *
 * 设置运行时地图实体的朝向。
 * 参数：entity_id 为运行时地图实体 ID；facing 为引擎朝向值。
 * 实体不存在或朝向未变时不操作；朝向改变时刷新“当前动画组起点 + 朝向”，
 * 不替换动画组。原生 setter 存储的是字节，并非经过校验的两位枚举；应使用
 * FACING_DOWN/UP/LEFT/RIGHT，不要指望引擎拒绝域外数值。四版 SetAnimFacing
 * 函数体一致。
 */
void SetEntityFacing(MaryEntityId entity_id, MaryFacingDirection facing);

/*
 * Returns a runtime map entity's facing direction.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the stored direction byte, or 0 if the entity does not exist.
 * Consequently FACING_DOWN alone does not prove that the entity exists.
 *
 * 返回运行时地图实体的朝向。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：已存储的朝向字节；实体不存在时也返回 0。因此单凭 FACING_DOWN
 * 不能证明实体存在。
 */
MaryFacingDirection GetEntityFacing(MaryEntityId entity_id);

/*
 * Sets a runtime entity's default OBJ display priority; it does not seat or
 * release an actor. Parameters: entity_id selects the scene entity; priority
 * is stored in actor byte +0x21 and its low two bits feed the sprite renderer.
 * The four native leaves store the byte without validating it and do not
 * immediately refresh the entity. Rendering masks it with 3, so the public
 * MaryEntitySpritePriority domain exposes the four physically distinct values.
 * Tea-party scripts use 1 at the table and restore 2 afterward to change
 * occlusion. The visual controller can override this default with a per-piece
 * priority mapping, so it is not an unconditional "draw in front" command.
 *
 * 设置运行时实体的默认 OBJ 显示优先级，并不使角色就座或离座。
 * 参数：entity_id 选择场景实体；priority 写入角色 +0x21 字节，绘制时取低两位。
 * 四版原生叶函数均不验证数值，只写入低字节，也不会立即刷新实体；绘制端再与 3
 * 相与，因此公开的 MaryEntitySpritePriority 域列出四个物理上不同的值。
 * 茶会脚本在桌边使用 1，结束后恢复 2，以改变遮挡关系。视觉控制器可以用逐部件
 * 优先级映射覆盖此默认值，因此它不是无条件的“显示在最前”命令。
 */
void SetEntitySpritePriority(MaryEntityId entity_id, MaryEntitySpritePriority priority);

/*
 * Starts horizontal movement toward an absolute X coordinate.
 * Parameters: entity_id selects the runtime scene entity; x is the target
 * coordinate; speed is a MaryEntityMoveSpeed measured in pixels per frame and
 * converted by the engine to Q16.16. The call starts movement and returns immediately; use
 * WaitForEntityMovement when synchronization is required.
 * X/Y and Raw variants share one movement command per entity; a later call
 * replaces the previous axis/target rather than combining into diagonal motion.
 * Setup discards the selected coordinate's fractional part. Zero speed or a
 * target equal to that integer coordinate clears the movement command;
 * otherwise the speed magnitude is used and the target is stored as u16.
 * Negative target coordinates therefore do not mean movement outside the map:
 * they wrap in the native target field. No source-bytecode normalization is done.
 * Evidence: the paired native setup bodies at FoMT-US 0x08032308,
 * FoMT-JP 0x0803209C, MFoMT-US 0x080326C4 and MFoMT-JP 0x08032538 agree.
 *
 * 启动实体向绝对 X 坐标的水平移动。
 * 参数：entity_id 为运行时场景实体编号；x 为目标坐标；speed 为以每帧像素数
 * 表示的 MaryEntityMoveSpeed，引擎会将其转换成 Q16.16。本调用只启动移动并立即返回；需要同步时应再调用
 * WaitForEntityMovement。
 * X/Y 及 Raw 变体共用每个实体唯一的移动命令；后一次调用覆盖前一次的轴与目标，
 * 不会组合成斜向移动。设置时丢弃所选坐标的小数部分；速度为零或目标等于该整数
 * 坐标时，清除移动命令；否则使用速度的绝对值，并将目标存入 u16 字段。
 * 因此负目标坐标并不表示向地图外移动，而会在原生目标字段中回绕；编译器不会
 * 因此擅自改写源字节码。证据为四版相同的成对设置函数：FoMT-US 0x08032308、
 * FoMT-JP 0x0803209C、MFoMT-US 0x080326C4、MFoMT-JP 0x08032538。
 */
void MoveEntityXTo(MaryEntityId entity_id, MaryMapSpaceX x, MaryEntityMoveSpeed speed);

/*
 * Starts horizontal movement like MoveEntityXTo.
 * Parameters: entity_id is the
 * runtime scene entity; x is the absolute target coordinate; speed_q16 is a
 * MaryEntityMoveSpeedQ16 passed directly without scaling.
 * The shared-command, zero-speed and u16-target rules of MoveEntityXTo apply.
 *
 * 与 MoveEntityXTo 相同地启动水平移动。
 * 参数：entity_id 为运行时场景实体；
 * x 为绝对目标坐标；speed_q16 为 MaryEntityMoveSpeedQ16，会直接传入而不做缩放。
 * 同样遵循 MoveEntityXTo 的共用命令、零速度及 u16 目标字段规则。
 */
void MoveEntityXToRaw(MaryEntityId entity_id, MaryMapSpaceX x, MaryEntityMoveSpeedQ16 speed_q16);

/*
 * Starts vertical movement toward an absolute Y coordinate.
 * Parameters: entity_id selects the runtime scene entity; y is the target
 * coordinate; speed is a MaryEntityMoveSpeed measured in pixels per frame and
 * converted by the engine to Q16.16. Use WaitForEntityMovement when synchronization is required.
 * Shares the command slot and target-storage rules documented for MoveEntityXTo.
 *
 * 启动实体向绝对 Y 坐标的垂直移动。
 * 参数：entity_id 为运行时场景实体编号；y 为目标坐标；speed 为以每帧像素数
 * 表示的 MaryEntityMoveSpeed，引擎会将其转换成 Q16.16。需要同步时应再调用 WaitForEntityMovement。
 * 与 MoveEntityXTo 共用移动命令，并遵循其目标存储规则。
 */
void MoveEntityYTo(MaryEntityId entity_id, MaryMapSpaceY y, MaryEntityMoveSpeed speed);

/*
 * Starts vertical movement like MoveEntityYTo.
 * Parameters: entity_id is the
 * runtime scene entity; y is the absolute target coordinate; speed_q16 is a
 * MaryEntityMoveSpeedQ16 passed directly without scaling.
 * The shared-command, zero-speed and u16-target rules of MoveEntityXTo apply.
 *
 * 与 MoveEntityYTo 相同地启动垂直移动。
 * 参数：entity_id 为运行时场景实体；
 * y 为绝对目标坐标；speed_q16 为 MaryEntityMoveSpeedQ16，会直接传入而不做缩放。
 * 同样遵循 MoveEntityXTo 的共用命令、零速度及 u16 目标字段规则。
 */
void MoveEntityYToRaw(MaryEntityId entity_id, MaryMapSpaceY y, MaryEntityMoveSpeedQ16 speed_q16);

/*
 * Suspends the current script until the selected scene entity finishes its
 * active movement command.
 * Parameter: entity_id addresses a live scene entity,
 * in the same domain as SetEntityPosition and SetEntityFacing. Scripts may
 * call this once for each entity after starting simultaneous movement.
 *
 * 暂停当前脚本，直到指定场景实体完成正在执行的移动命令。
 * 参数：entity_id 为运行时场景实体编号，与 SetEntityPosition、
 * SetEntityFacing 使用同一编号域。脚本同时启动多个实体移动后，可以依次对
 * 每个实体调用本函数。
 */
void WaitForEntityMovement(MaryEntityId entity_id);

/*
 * Sets a runtime map entity's direction-relative animation group.
 * Parameters: entity_id is the runtime map entity ID; animation_id is the
 * group's base ID in the target-specific ANIMATION_ID_* domain, not an
 * already direction-adjusted frame/animation ID. AActorEntity::SetAnim stores
 * the base and refreshes base + current facing. SetEntityAnim skips this call
 * when the entity is absent or already has that base, so repeating it does
 * not restart the animation. SetEntityFacing changes the directional offset.
 * Native SetAnim entries are FoMT-US 0x080321B0, FoMT-JP 0x08031F44,
 * MFoMT-US 0x0803256C, MFoMT-JP 0x080323E0; their bodies are byte-identical.
 * Not every physical ANIMATION_ID_* is a suitable group base. The compiler
 * preserves the supplied integer and does not silently align it to a group.
 *
 * 设置运行时地图实体随朝向变化的动画组。
 * 参数：entity_id 为运行时地图实体 ID；animation_id 是目标版本 ANIMATION_ID_*
 * 域内的动画组起点，不是已经加过朝向偏移的动画编号或帧编号。
 * AActorEntity::SetAnim 保存组起点，再刷新“起点 + 当前朝向”。实体不存在或
 * 已经使用该组时，SetEntityAnim 不调用刷新，因此重复设置不会重启动画；
 * SetEntityFacing 则改变朝向偏移。四版 SetAnim 原生入口依次为 0x080321B0、
 * 0x08031F44、0x0803256C、0x080323E0，其函数体逐字节一致。
 * 并非任意物理 ANIMATION_ID_* 都适合作为组起点；编译器原样保留输入整数，
 * 不会擅自将其对齐到某个动画组。
 */
void SetEntityAnim(MaryEntityId entity_id, MaryAnimationId animation_id);

/*
 * Compatibility no-op at callable slot 0x00E. The VM consumes one integer.
 * With no current script entity it exits directly; otherwise all four native
 * FoMT/MFoMT US/JP implementations call an empty function that immediately
 * returns, then exit. It is exposed so otherwise-unused binary input containing
 * this slot can still round-trip without an unknown call.
 * Parameter: unused_value is preserved only as the observed stack operand;
 * the native no-op does not inspect it.
 *
 * callable 槽 0x00E 的兼容空操作。VM 会取出一个整数；不存在当前脚本实体时
 * 直接退出，否则 FoMT/MFoMT 的 US/JP 四版原生实现会调用一个立即返回的空函数，
 * 随后退出。公开它仅用于让包含该槽的二进制输入仍能无损往返，而不是建议新脚本使用。
 * 参数：unused_value 仅用于保留观察到的栈操作数；原生空操作不会检查它。
 */
void NoOp014(int unused_value);

/*
 * Hides a live scene entity by moving it to the engine's reserved off-map
 * location.
 * Parameter: entity_id uses the runtime scene-entity domain. This
 * does not destroy the entity; a later position/map operation may show it
 * again.
 *
 * 通过把运行时场景实体移到引擎保留的离场地点来隐藏它。
 * 参数：entity_id 使用运行时场景实体编号域。本操作不会销毁实体；后续位置或
 * 地图操作仍可让它再次出现。
 */
void HideEntity(MaryEntityId entity_id);

/*
 * Selects the auxiliary render component used by a temporary event entity.
 * entity_id selects the runtime entity through the scene's virtual entity
 * lookup; render_profile is written unchanged to offset 0x88 of its visual
 * controller. The native renderer maps values 0, 1, and 2 to auxiliary
 * components 0, 1, and 2, while value 3 suppresses that component. Retail
 * scripts use profiles 0, 2, and 3 for small animals, livestock, and human
 * actors respectively; native entity constructors also use profile 1.
 * Parameters: entity_id selects the runtime entity; render_profile selects
 * AUX_RENDER_PROFILE_* and is stored unchanged.
 *
 * 选择临时事件实体使用的辅助渲染组件。entity_id 会经场景虚函数查询选中
 * 运行时实体；render_profile 原样写入其视觉控制器偏移 0x88。原生渲染器把
 * 0、1、2 分别映射到辅助组件 0、1、2，值 3 则跳过该组件。原版脚本分别将
 * 配置 0、2、3 用于小型动物、家畜和人物角色；原生实体构造器也会使用配置 1。
 * 参数：entity_id 选择运行时实体；render_profile 选择 AUX_RENDER_PROFILE_*，
 * 并会原样保存。
 */
void SetEntityAuxRenderProfile(MaryEntityId entity_id, MaryEntityAuxRenderProfile render_profile);

/*
 * Starts an auxiliary visual effect attached to a live scene entity.
 * Parameters: entity_id selects the entity; effect_id selects the engine
 * emote bubble; persistent is zero for the ordinary finite form and nonzero
 * for the persistent form. effect_id uses ENTITY_EMOTE_* or the exact numeric
 * resource index.
 * The ordinary form clears on the animator's first wrap notification (bit 2),
 * not after a fixed frame count. Persistent mode skips that automatic clear;
 * it does not override the animation's timing. A zero-duration frame or a
 * stopped animator can prevent wrapping, so use StopEntityEffect when explicit
 * cleanup is required. FoMT-US reads this status at 0x08032670; the matching
 * JP/MFoMT paths and animation stepper bodies are byte-identical.
 *
 * 在运行时场景实体上启动一个附加视觉效果。
 * 参数：entity_id 选择实体；effect_id 使用 ENTITY_EMOTE_* 或精确数字资源
 * 索引来选择表情气泡；persistent 为零时使用普通有限时长形式，非零时使用
 * 持续形式。
 * 普通形式在动画器首次发出回绕通知（第 2 位）时清除，并非经过固定帧数后清除。
 * 持续模式跳过此自动清除，但不覆盖动画本身的时序。零时长帧或停止的动画器
 * 可能不会发生回绕；需要明确清理时应调用 StopEntityEffect。FoMT-US 在
 * 0x08032670 检查该状态，其余三版对应路径和动画步进函数体均逐字节一致。
 */
void StartEntityEffect(MaryEntityId entity_id, MaryEntityEmoteId effect_id, MaryBool persistent);

/*
 * Stops and clears the auxiliary visual effect attached to an entity.
 * Parameter: entity_id selects the same runtime entity domain used by
 * StartEntityEffect. The independently decompiled FoMT-US, FoMT-JP,
 * MFoMT-US, and MFoMT-JP slot-0x012 handlers all resolve the entity and clear
 * bits 0-1 at offset 0x8A of its visual controller. They neither poll nor
 * suspend the script, so the `Wait_For_Animation` label in FOMT Studio's
 * reference CSV is not the native behavior.
 *
 * 停止并清除附加在实体上的视觉效果。
 * 参数：entity_id 与 StartEntityEffect 使用相同的运行时实体编号域。对 FoMT-US、
 * FoMT-JP、MFoMT-US、MFoMT-JP 的 0x012 槽 handler 独立反编译后，四者都会解析
 * 实体并清除其视觉控制器偏移 0x8A 的第 0-1 位；它们既不轮询状态，也不暂停
 * 脚本。因此 FOMT Studio 参考 CSV 中的 `Wait_For_Animation` 标签并非原生行为。
 */
void StopEntityEffect(MaryEntityId entity_id);

/*
 * Converts a facing direction to its opposite direction.
 * Parameter: facing is the engine direction value.
 * Return value: DOWN maps to UP, UP to DOWN, LEFT to RIGHT, and RIGHT to LEFT.
 * All four native handlers return FACING_UP for an out-of-domain input rather
 * than preserving or rejecting it; callers should still pass MaryFacingDirection.
 *
 * 将朝向转换为相反方向。
 * 参数：facing 为引擎朝向值。
 * 返回值：下转为上、上转为下、左转为右、右转为左。四版原生 handler 对域外
 * 输入都会返回 FACING_UP，而不是保留原值或拒绝输入；调用方仍应传入
 * MaryFacingDirection 域内值。
 */
MaryFacingDirection GetOppositeFacing(MaryFacingDirection facing);

/*
 * Returns the map containing a runtime map entity. FoMT source calls this
 * value `location`, and all four ROM handlers return the same map-ID field
 * consumed by ChangeMap and the fireplace callables.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the entity's MaryMapId, or MAP_NONE for an absent entity.
 * MAP_NONE is 0x0234 in FoMT and 0x023A in MFoMT; it can also identify a
 * detached/hidden entity, so it is not a unique missing-entity indicator.
 *
 * 返回运行时地图实体所在的地图。FoMT 源码将该值称为 `location`；四个 ROM
 * 的处理函数均返回与 ChangeMap 及壁炉函数共用的地图 ID 字段。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体所在地图的 MaryMapId；实体不存在时返回 MAP_NONE（FoMT 为
 * 0x0234，MFoMT 为 0x023A）。脱离地图／隐藏的实体也可以具有该值，因此它
 * 不是“实体不存在”的唯一标志。
 */
MaryMapId GetEntityLocation(MaryEntityId entity_id);

/*
 * Moves a live scene entity relative to its current position.
 * Parameters:
 * entity_id selects the runtime entity; delta_x and delta_y are signed
 * map-space offsets added to its current coordinates. This is distinct from
 * SetEntityPosition, whose coordinates are absolute. The four native leaves
 * (FoMT-US 0x0801223C, FoMT-JP 0x0801210C, MFoMT-US 0x0801232C, and MFoMT-JP
 * 0x080121D0) resolve the entity, read its current signed X and Y fields at
 * offsets 0x0A and 0x0E, add delta_x and delta_y, and pass the resulting
 * coordinates to the entity-position updater. Matching FoMT/MFoMT US/JP
 * wedding scripts use it to shift participants by one tile. A FoMT debug
 * procession passes player-coordinate expressions as deltas; that unusual
 * call site does not change the native relative-coordinate contract.
 *
 * 按当前位置相对移动运行时场景实体。
 * 参数：entity_id 选择运行时实体；delta_x、delta_y 是分别加到当前坐标上的
 * 有符号地图空间偏移量。本函数不同于使用绝对坐标的 SetEntityPosition。
 * 四版原生叶函数（FoMT-US 0x0801223C、FoMT-JP 0x0801210C、MFoMT-US
 * 0x0801232C、MFoMT-JP 0x080121D0）都会解析实体，读取其偏移 0x0A、0x0E 的
 * 当前有符号 X、Y 字段，分别加上 delta_x、delta_y，再把结果交给实体位置更新
 * 函数。FoMT/MFoMT 的 US/JP 对应婚礼脚本都会用它把参与者平移一格。FoMT 的
 * 调试游行脚本把玩家坐标表达式作为偏移量传入；这一特殊调用点不会改变原生函数
 * 的相对坐标契约。
 */
void OffsetEntityPosition(MaryEntityId entity_id, MaryMapSpaceX delta_x, MaryMapSpaceY delta_y);

/*
 * Changes the active field map and installs the destination coordinates in
 * the engine's packed location record.
 * Parameters: map_id is a MaryMapId or
 * the exact target-specific numeric map ID; x and y are destination map-space
 * coordinates local to that map. The native packed Location keeps the low
 * 10 bits of map_id and the low 16-bit two's-complement representation of
 * each coordinate; it performs no range rejection. Scripts normally fade out first and then set the player
 * entity's position/facing explicitly after this call.
 *
 * 切换当前野外地图，并把目标坐标写入引擎的压缩位置记录。
 * 参数：map_id 为 MaryMapId 或目标版本的精确数字地图 ID；x、y 为目标地图
 * 内部坐标，不是跨地图的全局坐标。原生压缩 Location 保留 map_id 的低 10 位，
 * 以及两个坐标各自低 16 位的补码表示，不执行越界拒绝。脚本通常先淡出画面，
 * 调用本函数后再显式设置玩家实体的位置和朝向。
 */
void ChangeMap(MaryMapId map_id, MaryMapSpaceX x, MaryMapSpaceY y);

/*
 * Starts an event-camera pan toward a map coordinate.
 * Parameters: x and y are the camera target in map space; speed is the integer
 * movement speed used by the field controller. Shipped scripts use values
 * including 1, 2, and 5, so this is a continuous speed quantity rather than a
 * closed selector enum. This does not move the player
 * entity. Scripts commonly start matching movement for entity 0 separately,
 * then pan the camera to the same destination. Use WaitForCameraMovement when
 * later commands must wait for the pan itself.
 * The native camera setup treats speed 0 as 1, not as a stop command.
 * x/y request the viewport center, not its top-left corner. The engine
 * subtracts (120, 80), then bounds the resulting viewport origin using map
 * width/height minus (240, 160). Near map edges the requested point therefore
 * need not appear at the screen center. These remain map-space coordinates.
 * The controller derives an integer update count from distance/speed (at
 * least one for a nonzero integer displacement), then divides the fixed-point
 * displacement by that count. Integer rounding means speed is not an exact
 * per-update displacement guarantee. A new pan replaces the stored increments
 * and count; it is not appended to a movement queue.
 *
 * 启动事件镜头向指定地图坐标平移。
 * 参数：x、y 为地图空间中的镜头目标；speed 为场景控制器采用的整数移动速度。
 * 原版脚本使用的值包括 1、2、5，因此它是连续速度量，而非封闭的选择枚举。
 * 本函数不会移动玩家实体。脚本经常另外启动实体 0 的对应移动，再让
 * 镜头以同一目标同步平移。后续指令必须等待镜头本身完成时，应调用
 * WaitForCameraMovement。
 * 原生镜头初始化会将速度 0 按 1 处理，0 不是停止命令。
 * x/y 指定期望的视口中心，而非左上角。引擎先减去 (120, 80)，再根据地图
 * 宽高减去 (240, 160) 限制视口起点；因此靠近地图边缘时，指定点不一定能
 * 出现在屏幕正中央。这些参数仍属于地图空间坐标。
 * 控制器由距离／速度求出整数更新次数（整数位移非零时至少为一次），再以
 * 定点位移除以次数得到增量。因此受整数取整影响，speed 不保证每次更新都
 * 精确移动该距离。新的平移会覆盖已存储的增量与计数，不会追加到移动队列。
 */
void PanCameraTo(MaryMapSpaceX x, MaryMapSpaceY y, MaryCameraMoveSpeed speed);

/*
 * Suspends the current script until the field controller reports that the
 * active event-camera pan has finished. It does not wait for an entity's
 * movement; use WaitForEntityMovement for that independent state.
 * Completion means the camera's remaining-update counter has reached zero,
 * not an exact coordinate comparison. Each camera update adds the stored
 * Q16.16 X/Y increments and decrements this counter.
 * Parameters: none.
 *
 * 暂停当前脚本，直到场景控制器报告正在执行的事件镜头平移已经完成。它不会
 * 等待实体移动；后者是由 WaitForEntityMovement 独立等待的状态。
 * 完成条件是镜头的剩余更新计数归零，并非精确比较坐标。每次镜头更新会
 * 累加已存储的 Q16.16 X/Y 增量，并将该计数减一。
 * 参数：无。
 */
void WaitForCameraMovement(void);

/*
 * Starts background music through the event state's dedicated BGM player.
 * Parameters: start_mode selects AUDIO_START, AUDIO_START_WEAK, or
 * AUDIO_START_OR_CONTINUE; song_id is the m4a song-table sequence ID.
 * The native wrapper truncates song_id to 16 bits. It then applies the
 * event handler's neutral volume, pitch, pan, and modulation settings to
 * this player; it does not allocate from the PlaySong player pool.
 *
 * 通过事件状态中的专用 BGM 播放器播放背景音乐。
 * 参数：start_mode 选择 AUDIO_START、AUDIO_START_WEAK 或
 * AUDIO_START_OR_CONTINUE；song_id 为 m4a 曲目表中的序列 ID。
 * 原生 wrapper 会将 song_id 截断为 16 位，随后把事件 handler 提供的中性
 * 音量、音高、声像及调制参数应用到该播放器；它不会从 PlaySong 播放器池分配。
 */
void PlayBGM(MaryAudioStartMode start_mode, MaryAudioSequenceId song_id);

/*
 * Stops the event state's dedicated background-music player immediately.
 * It does not stop the independent PlaySong pool; use StopAllSongs for the
 * global five-player stop operation.
 * Parameters: none.
 *
 * 立即停止事件状态中的专用背景音乐播放器。它不会停止独立的 PlaySong
 * 播放器池；若需执行全局五播放器停止操作，应使用 StopAllSongs。
 * 参数：无。
 */
void StopBGM(void);

/*
 * Starts a song or sound-effect sequence through an available event
 * MusicPlayer.
 * Parameters: start_mode selects AUDIO_START,
 * AUDIO_START_WEAK, or AUDIO_START_OR_CONTINUE; sequence_id is the m4a
 * song-table sequence ID.
 * The native PlaySong handler keeps only the low 16 bits of sequence_id.
 * This truncation is not a song-table bounds check; use a sequence present
 * in the selected ROM. Mary-C preserves the original integer in bytecode.
 * The handler chooses the first stopped player in its pool. If none is
 * stopped, it applies the requested start mode to the last player instead;
 * there is no request queue or automatic wait for a free player.
 *
 * 通过事件可用的 MusicPlayer 播放歌曲或音效序列。
 * 参数：start_mode 选择 AUDIO_START、AUDIO_START_WEAK 或
 * AUDIO_START_OR_CONTINUE；sequence_id 为 m4a 曲目表中的序列 ID。
 * 原生 PlaySong handler 只保留 sequence_id 的低 16 位。这不是曲目表边界
 * 检查，仍应使用目标 ROM 中实际存在的序列。Mary-C 在字节码中保留原始整数。
 * handler 优先选择池中第一个已停止的播放器；若全都未停止，则对最后一个
 * 播放器执行所选启动模式。没有请求队列，也不会自动等待播放器空闲。
 */
void PlaySong(MaryAudioStartMode start_mode, MaryAudioSequenceId sequence_id);

/*
 * Stops all active songs or sound sequences.
 * This calls the global m4a player-table stop operation, not just the event
 * PlaySong pool. It can stop background music as well; use StopBGM when only
 * the dedicated background-music player should be stopped.
 * Parameters: none.
 *
 * 停止所有正在播放的歌曲或声音序列。
 * 本函数调用全局 m4a 播放器表的停止操作，不只处理事件 PlaySong 播放器池，
 * 因而也可能停止背景音乐。若只需停止专用背景音乐播放器，应使用 StopBGM。
 * 参数：无。
 */
void StopAllSongs(void);

/*
 * Fades out the current background music.
 * Requests the m4a fade with fixed speed parameter 5 only when the dedicated
 * BGM player is not stopped. The script continues without waiting for the
 * fade to finish; 5 is an engine fade parameter, not a duration in seconds.
 * The m4a parameter reloads the countdown between volume steps. A repeated
 * accepted request reinitializes this countdown and the fade volume; it does
 * not simply leave an existing fade untouched.
 * An uninterrupted fade decreases the internal level from 64 by 4 every
 * five FadeOutBody updates: 16 steps (80 such updates), then track stop and
 * player pause. These are updater calls, not a wall-clock timing guarantee.
 * Parameters: none.
 *
 * 淡出当前背景音乐。
 * 仅在专用 BGM 播放器未停止时，以固定速度参数 5 请求 m4a 淡出。脚本不会
 * 等待淡出完成；5 是引擎淡出参数，不是以秒计的持续时间。
 * 此 m4a 参数用于重装音量步进之间的倒计时。再次被接受的请求会重新初始化
 * 倒计时及淡出音量，并非保持正在进行的淡出不变。
 * 未被中途重置时，内部音量从 64 开始，每五次 FadeOutBody 更新降低 4，
 * 共十六级（八十次该函数更新），然后停止音轨并暂停播放器。此处是更新
 * 函数调用次数，不保证实际经过的墙钟时间。
 * 参数：无。
 */
void FadeOutBGM(void);

/*
 * Destroys and clears the talk UI's auxiliary components and active text
 * window, then clears the corresponding active-state bit. This is a stronger
 * reset than TalkClose(). The four vanilla script sets contain no direct
 * call to this callable, but selected VarSet event-state transitions invoke
 * the same native cleanup internally; absence of a script call does not mean
 * this cleanup is unused. Parameters: none.
 *
 * 销毁并清空对话界面的辅助组件及当前文本窗口，同时清除对应的活动状态位。
 * 本操作比 TalkClose() 更彻底。四套原版脚本均无对此 callable 的直接调用，
 * 但部分 VarSet 事件状态变化会在内部调用相同的原生清理函数，不能据此认定
 * 该清理逻辑未使用。参数：无。
 */
void ResetTalkUi(void);

/*
 * Opens the dialogue window. The native routine first destroys and clears the
 * pause-menu page selector/header plus the composite Time Window HUD.
 * It creates the text window only when none
 * exists, otherwise it reuses the existing object and
 * requests its opening state. Call this before TalkMessage or TalkChoiceN.
 * Unlike ResetTalkUi, it does not destroy or null an existing text window.
 * Parameters: none.
 *
 * 打开对话框。原生函数会先销毁并清空暂停菜单页签选择／标题组件，以及一个与
 * Time Window HUD；仅在
 * 文本窗口不存在时创建新对象，否则复用现有对象并请求进入打开状态。应在
 * TalkMessage 或 TalkChoiceN 之前调用。与 ResetTalkUi 不同，本函数不会销毁
 * 或置空已有文本窗口。
 * 参数：无。
 */
void TalkOpen(void);

/*
 * Opens MFoMT's dialogue window without a portrait or named speaker. The
 * native handler first removes the same pause-menu page selector/header and
 * composite Time Window HUD,
 * then unconditionally creates a fresh text-window component and
 * replaces any existing text window. Unlike TalkOpen, it does not reuse an
 * already allocated text window. Use this for narration and other speakerless
 * messages; ordinary character dialogue uses TalkOpen().
 * Parameters: none.
 *
 * 打开 MFoMT 的无头像、无说话者姓名对话框。原生处理函数会先移除上述暂停菜单
 * 页签选择／标题组件和复合 Time Window HUD，再无条件创建新的文本窗口，并替换已有文本窗口；它不像
 * TalkOpen 那样复用已经分配的文本窗口。旁白及其他无说话者文本使用本函数；
 * 普通人物对话使用 TalkOpen()。
 * 参数：无。
 */
void TalkOpenNoPortrait(void);

/*
 * Closes the dialogue window opened by TalkOpen.
 * With no text-window object, the native close routine does nothing. Otherwise
 * it requests the closing state and clears internal window flags; unlike
 * ResetTalkUi, this routine does not destroy and null the window object.
 * Parameters: none.
 *
 * 关闭由 TalkOpen 打开的对话框。
 * 没有文本窗口对象时，原生关闭函数不执行操作；否则请求进入关闭状态并清理
 * 窗口内部标志。与 ResetTalkUi 不同，该函数不会销毁并置空窗口对象。
 * 参数：无。
 */
void TalkClose(void);

/*
 * Submits a message segment to the open dialogue window using the native
 * text-rate value 0x0100, then requests dialogue state 0x10.
 * Parameter: message is a text symbol from the current script's mary_text_table.
 *
 * 以原生文本速率值 0x0100 向已打开的对话框提交一个文本段，随后请求对话状态
 * 0x10。
 * 参数：message 为当前脚本 mary_text_table 中的文本符号。
 */
void TalkMessage(const char *message);

/*
 * Submits a message segment using the engine's slower native text-rate value
 * 0x0040, then requests the same dialogue state 0x10 as TalkMessage.
 * Parameter: message is a text symbol from the current script's mary_text_table.
 *
 * 以引擎较慢的原生文本速率值 0x0040 提交文本段，随后请求与 TalkMessage 相同的
 * 对话状态 0x10。
 * 参数：message 为当前脚本 mary_text_table 中的文本符号。
 */
void TalkMessageSlow(const char *message);

/*
 * Submits an immediately completed text segment to the currently open
 * dialogue using native text-rate value 0. Vanilla scripts use the resulting
 * visible continuation to compose displays from separately selected segments.
 * This does not concatenate or mutate C string storage.
 * Parameter: message is a text symbol from the current script. This
 * is used to build displays such as a date followed by a separately selected
 * time string.
 *
 * 以原生文本速率值 0 向当前已打开的对话框提交立即完成的文本段。原版脚本利用
 * 其可见的连续显示效果，把分别选择的文本段组合起来；本函数不会在内存中拼接或
 * 修改 C 字符串。
 * 参数：message 为当前脚本中的文本符号。日期后再追加按条件选择的时间字符串
 * 等组合显示会使用本函数。
 */
void TalkAppendMessage(const char *message);

/*
 * Displays a prompt followed by two choices.
 * Parameters: prompt is the leading message; option_1 and option_2 are shown
 * in order. Return value: the selected one-based option number (1 or 2).
 *
 * 显示一段提示文本以及两个选项。
 * 参数：prompt 为前置提示；option_1、option_2 按顺序显示。
 * 返回值：从 1 开始的选项编号（1 或 2）。
 */
MaryPromptChoiceResult TalkPromptChoice2(
    const char *prompt,
    const char *option_1,
    const char *option_2
);

/*
 * Displays a prompt followed by three choices.
 * Parameters: prompt is the leading message; option_1 through option_3 are
 * shown in order. Return value: the selected one-based option number (1 to 3).
 *
 * 显示一段提示文本以及三个选项。
 * 参数：prompt 为前置提示；option_1 至 option_3 按顺序显示。
 * 返回值：从 1 开始的选项编号（1 至 3）。
 */
MaryPromptChoiceResult TalkPromptChoice3(
    const char *prompt,
    const char *option_1,
    const char *option_2,
    const char *option_3
);

/*
 * Displays a prompt followed by four choices.
 * Parameters: prompt is the leading message; option_1 through option_4 are
 * shown in order. Return value: the selected one-based option number (1 to 4).
 *
 * 显示一段提示文本以及四个选项。
 * 参数：prompt 为前置提示；option_1 至 option_4 按顺序显示。
 * 返回值：从 1 开始的选项编号（1 至 4）。
 */
MaryPromptChoiceResult TalkPromptChoice4(
    const char *prompt,
    const char *option_1,
    const char *option_2,
    const char *option_3,
    const char *option_4
);

/*
 * Displays two choices.
 * Parameters: choice_1 and choice_2 are text symbols in display order.
 * Return value: the selected one-based option number (1 to 2).
 *
 * 显示两个选项。
 * 参数：choice_1、choice_2 为按显示顺序排列的文本符号。
 * 返回值：从 1 开始的选项编号（1 至 2）。
 */
MaryChoiceResult TalkChoice2(const char *choice_1, const char *choice_2);

/*
 * Displays three choices.
 * Parameters: choice_1 through choice_3 are text symbols in display order.
 * Return value: the selected one-based option number (1 to 3).
 *
 * 显示三个选项。
 * 参数：choice_1 至 choice_3 为按显示顺序排列的文本符号。
 * 返回值：从 1 开始的选项编号（1 至 3）。
 */
MaryChoiceResult TalkChoice3(const char *choice_1, const char *choice_2, const char *choice_3);

/*
 * Displays four choices.
 * Parameters: choice_1 through choice_4 are text symbols in display order.
 * Return value: the selected one-based option number (1 to 4).
 *
 * 显示四个选项。
 * 参数：choice_1 至 choice_4 为按显示顺序排列的文本符号。
 * 返回值：从 1 开始的选项编号（1 至 4）。
 */
MaryChoiceResult TalkChoice4(const char *choice_1, const char *choice_2, const char *choice_3, const char *choice_4);

/*
 * Displays five choices.
 * Parameters: choice_1 through choice_5 are text symbols in display order.
 * Return value: the selected one-based option number (1 to 5).
 *
 * 显示五个选项。
 * 参数：choice_1 至 choice_5 为按显示顺序排列的文本符号。
 * 返回值：从 1 开始的选项编号（1 至 5）。
 */
MaryChoiceResult TalkChoice5(const char *choice_1, const char *choice_2, const char *choice_3, const char *choice_4, const char *choice_5);

/*
 * Displays six choices.
 * Parameters: choice_1 through choice_6 are text symbols in display order.
 * Return value: the selected one-based option number (1 to 6).
 *
 * 显示六个选项。
 * 参数：choice_1 至 choice_6 为按显示顺序排列的文本符号。
 * 返回值：从 1 开始的选项编号（1 至 6）。
 */
MaryChoiceResult TalkChoice6(const char *choice_1, const char *choice_2, const char *choice_3, const char *choice_4, const char *choice_5, const char *choice_6);

/*
 * Displays a character's localized name in the talk-window nameplate.
 * Parameter: character_id is CHARACTER_* or the exact target-specific numeric
 * ID. The handler resolves the ID through the character-name table, then uses
 * the same nameplate renderer as SetTalkNameplateText. It does not store a
 * persistent speaker or interaction-character identity. Use SetTalkPortrait
 * separately to select the portrait and expression. Both nameplate setters
 * leave the nameplate unchanged when the player's "Name" display option is off.
 *
 * 在对话框姓名牌中显示指定人物的本地化姓名。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确数字 ID。处理函数先经
 * 人物姓名表把 ID 转成字符串，再进入与 SetTalkNameplateText 相同的姓名牌
 * 渲染函数；它不会保存持久的说话者或交互人物身份。头像及表情需另外通过
 * SetTalkPortrait 设置。玩家关闭“Name／姓名显示”选项时，两种姓名牌设置函数
 * 都保持当前姓名牌不变。
 */
void SetTalkNameplateCharacter(MaryCharacterId character_id);

/*
 * Displays caller-supplied text in the talk-window nameplate. The native
 * renderer accepts at most 12 encoded bytes and clears the nameplate when the
 * text is empty or too long. Choice menus use this slot for short headings
 * such as "Pick one"; it can also provide a name not present in the character
 * table. The text remains active until replaced or cleared with
 * ClearTalkNameplate(). Like SetTalkNameplateCharacter, it has no effect while
 * the player's "Name" display option is off. Parameter: text is a symbol from the
 * current script.
 *
 * 在对话框姓名牌中显示调用者提供的文字。原生渲染器最多接受 12 个编码字节；
 * 空字符串或过长字符串会清除姓名牌。选择菜单用该槽显示“Pick one”等短标题，
 * 也可用它显示人物姓名表中没有的名字。文字会一直保留，直到被替换或由
 * ClearTalkNameplate() 清除。与 SetTalkNameplateCharacter 相同，玩家关闭
 * “Name／姓名显示”选项时本函数不产生效果。参数：text 为当前脚本文本表中的符号。
 */
void SetTalkNameplateText(const char *text);

/*
 * Clears the talk-window nameplate without closing the message window or
 * changing the portrait. The native clear path is not blocked when the
 * player's "Name" display option is off.
 * Parameters: none.
 *
 * 清除对话框姓名牌，但不关闭消息窗口，也不改变头像；即使玩家关闭
 * “Name／姓名显示”选项，原生清除路径仍然执行。
 * 参数：无。
 */
void ClearTalkNameplate(void);

/*
 * Sets the portrait and expression shown by dialogue.
 * Parameter: portrait_id is TALK_PORTRAIT_* or the exact target-specific ID.
 * The native routine forwards the ID only while the player's "Face" display
 * option is on; otherwise it leaves the portrait unchanged. This operation uses the current text-window UI
 * object and does not implicitly set the independently rendered nameplate.
 * Use SetTalkNameplateCharacter separately to display the character's name.
 *
 * 设置对话显示的头像与表情。
 * 参数：portrait_id 为 TALK_PORTRAIT_* 或目标版本的精确 ID。
 * 原生函数仅在玩家开启“Face／头像显示”选项时转发该 ID，否则保持头像不变。
 * 本操作使用当前文本窗口 UI 对象，不会隐式设置独立渲染的姓名牌。
 * 人物姓名需另外通过 SetTalkNameplateCharacter 显示。
 */
void SetTalkPortrait(MaryTalkPortraitId portrait_id);

/*
 * Clears the portrait currently displayed by the talk UI. This does not close
 * the text box and does not change the independently rendered nameplate. The
 * native clear path still runs when the player's "Face" display option is off.
 * Parameters: none.
 *
 * 清除对话界面当前显示的头像。该操作不会关闭文本框，也不会改变独立渲染的
 * 姓名牌；即使玩家关闭“Face／头像显示”选项，原生清除路径仍然执行。
 * 参数：无。
 */
void ClearTalkPortrait(void);

/*
 * Shows the talk-window heart indicator for a marriage candidate.
 * Parameter: character_id is CHARACTER_* or the exact target-specific numeric ID.
 * The native handler reads that character's current love points and maps them
 * to the black, purple, blue, green, yellow, orange, or red heart level. It
 * does not store a persistent "current interaction character".
 *
 * 显示指定结婚候补人物的对话框爱心指示。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确数字 ID。
 * 原生处理函数会读取该人物当前爱情度，并映射为黑、紫、蓝、绿、黄、橙、红
 * 七档爱心；它不会保存一个持久的“当前交互人物”。
 */
void ShowTalkHeartIndicator(MaryCharacterId character_id);

/* Hides the talk-window heart indicator.
 * Parameters: none. The FoMT handler clears the heart-display enable flag;
 * the MFoMT US/JP handlers perform the equivalent operation.
 *
 * 隐藏对话框中的爱心指示。
 * 参数：无。FoMT 处理函数会清除爱心显示启用标志；MFoMT US/JP 处理函数执行
 * 等价操作。
 */
void HideTalkHeartIndicator(void);

/*
 * Starts a screen fade-out and waits for the engine transition state.
 * Parameters: fade_style selects black, white, either color with mosaic, or
 * mosaic alone; fade_speed selects an approximately 15-, 30-, or 60-frame
 * transition. Scene-transfer scripts normally pair this with FadeInScreen
 * using the same arguments.
 *
 * 开始屏幕渐隐，并等待引擎进入过渡状态。
 * 参数：fade_style 选择黑场、白场、两者各自附带马赛克，或纯马赛克；
 * fade_speed 选择约 15、30、60 帧的过渡速度。场景切换脚本通常会在完成状态
 * 修改后，以相同参数调用 FadeInScreen。
 */
void FadeOutScreen(MaryScreenFadeStyle fade_style, MaryScreenFadeSpeed fade_speed);

/*
 * Starts the screen fade-in that completes a scene transition.
 * Parameters: fade_style and fade_speed use the same domains as
 * FadeOutScreen. The engine follows the inverse palette-transition path.
 *
 * 开始完成场景过渡的屏幕渐显。
 * 参数：fade_style 与 fade_speed 的编号域与 FadeOutScreen 相同；引擎执行
 * 与渐隐相反的调色板过渡路径。
 */
void FadeInScreen(MaryScreenFadeStyle fade_style, MaryScreenFadeSpeed fade_speed);

/*
 * Starts the same inward palette transition as FadeInScreen but deliberately
 * skips the active-scene virtual hook that FadeInScreen invokes immediately
 * before starting the transition. It is therefore a distinct native variant,
 * not an alias. Parameters use the same fade_style and fade_speed domains.
 * Vanilla achievement and Harvest Goddess flash sequences call this variant
 * twice before FadeOutScreen; normal scene restoration uses FadeInScreen.
 * Parameters: fade_style and fade_speed use the same domains as FadeInScreen.
 *
 * 启动与 FadeInScreen 相同的向内调色板过渡，但有意跳过 FadeInScreen 在开始
 * 过渡前调用的当前场景虚函数。因此它是独立的原生变体，而不是别名。参数使用
 * 相同的 fade_style 与 fade_speed 取值域。原版成就及女神闪光演出会在
 * FadeOutScreen 前连续调用两次此变体；普通场景恢复则使用 FadeInScreen。
 * 参数：fade_style、fade_speed 使用与 FadeInScreen 相同的取值域。
 */
void FadeInScreenWithoutSceneHook(
    MaryScreenFadeStyle fade_style,
    MaryScreenFadeSpeed fade_speed);

/*
 * Suspends the current script for frame_count engine frames.
 * Parameter: frame_count is written to the event state as an unsigned 16-bit
 * counter; values outside 0-65535 are truncated to their low 16 bits by the
 * native handler.
 * The waiting state decrements before testing for zero. A low-halfword value
 * of zero therefore takes 65536 waiting-state updates, not zero; -1 takes
 * 65535. This is not an immediate-return primitive or a wall-clock timer.
 *
 * 将当前脚本暂停 frame_count 个引擎帧。
 * 参数：frame_count 会作为无符号 16 位计数器写入事件状态；原生处理函数会把
 * 超出 0-65535 的值截成低 16 位。
 * 等待状态先递减再判断是否为零。因此低 16 位为零时需要 65536 次等待状态
 * 更新才结束，而不是立即返回；-1 对应 65535 次。它不是立即返回操作，
 * 也不是按实际时间计时的定时器。
 */
void WaitFrames(MaryFrameCount frame_count);

/*
 * Calls another slot in the selected ROM's ordered script table.
 * Parameter: script_id is a SCRIPT_* symbol from fomt_scripts.mary.h or the
 * exact numeric script-table ID. Both forms compile to the same integer.
 * The native lookup does not bounds-check script_id before reading
 * script_table[script_id], so it must name a valid slot for the selected
 * target. The resolved RIFF/script is handed to the current event runner,
 * which changes its state to 2. This is a Mary VM operation, not a normal C
 * ABI function call: it accepts no user arguments and returns no value.
 *
 * 调用所选 ROM 有序脚本表中的另一个槽位。
 * 参数：script_id 为 fomt_scripts.mary.h 中的 SCRIPT_* 符号或精确数字脚本 ID；
 * 两种形式编译为同一个整数。
 * 原生查表在读取 script_table[script_id] 前不会检查上界，因此该值必须是所选
 * 目标中的有效槽位。解析出的 RIFF／脚本会交给当前事件运行器，并把运行状态
 * 改为 2。这是 Mary VM 操作，不是普通 C ABI 函数调用：不能传入用户参数，
 * 也没有返回值。
 */
void CallScript(MaryScriptId script_id);

/*
 * Writes a numeric value into a numbered text-substitution slot.
 * Parameters: variable_index selects the {VarN} destination and must be one
 * of TEXT_VARIABLE_1 through TEXT_VARIABLE_4; the native setter performs no
 * bounds check. value is the signed integer formatted by the text engine.
 * This callable is slot 0x038 in FoMT and 0x039 in MFoMT. It formats through
 * the regional width-zero formatter, then uses the same byte-limited setter
 * as SetTextVariableString.
 *
 * 将数值写入编号文本替换槽。
 * 参数：variable_index 选择 {VarN} 目标，必须是 TEXT_VARIABLE_1 至
 * TEXT_VARIABLE_4；原生 setter 不做越界检查。value 为文本引擎格式化的
 * 有符号整数。
 * 本 callable 在 FoMT 为槽 0x038，在 MFoMT 为槽 0x039。它先通过地区对应的
 * 零宽度格式化器生成文本，再使用与 SetTextVariableString 相同的字节限长 setter。
 */
void SetTextVariableNumber(MaryTextVariableSlot variable_index, int value);

/*
 * Stores a decimal number in a text substitution slot and formats it to a
 * requested field width.
 * Parameters: variable_index selects {Var1}, {Var2},
 * and so on; value is the signed integer to display; field_width is the exact
 * decimal field width when nonzero. Shorter values are left-padded with ASCII
 * spaces; longer values lose their most-significant digits and retain only
 * the requested number of low-order digits. A zero width disables padding
 * and truncation. The US and JP native formatters use different loops, but
 * neither validates an arbitrary caller-supplied width before indexing its
 * local work buffer. New scripts must keep field_width in the original 0-10
 * domain; this is a caller contract, not a runtime clamp. variable_index
 * must be TEXT_VARIABLE_1 through TEXT_VARIABLE_4 because the native setter
 * does not bounds-check the destination. Shipped scripts
 * use width 2 for dates and width 3 for stock counts.
 * This callable is slot 0x039 in FoMT and 0x03A in MFoMT.
 *
 * 将十进制数字写入文本替换槽，并按指定字段宽度格式化。
 * 参数：variable_index 选择 {Var1}、{Var2} 等槽位；value 为要显示的有符号
 * 整数；field_width 非零时是精确十进制字段宽度。位数不足会在左侧补 ASCII
 * 空格，位数超出则截去高位、只保留指定数量的低位；宽度 0 表示不补齐也不
 * 截断。US 与 JP 原生格式化器使用不同循环，但都不会在索引局部工作缓冲区之前
 * 校验调用方给出的任意宽度。新脚本必须维持原版 0-10 的宽度域；这是调用契约，
 * 不是运行时限幅。
 * variable_index 必须是 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4，因为原生 setter
 * 不检查目标槽是否越界。原版脚本中，日期使用宽度 2，库存数量使用宽度 3。
 * 本 callable 在 FoMT 为槽 0x039，在 MFoMT 为槽 0x03A。
 */
void SetTextVariableNumberFieldWidth(
    MaryTextVariableSlot variable_index,
    int value,
    MaryTextNumberFieldWidth field_width
);

/*
 * Writes a string into a numbered text-substitution slot.
 * Parameters: variable_index must be TEXT_VARIABLE_1 through TEXT_VARIABLE_4;
 * the native setter performs no bounds check. text is copied into the slot
 * and truncated before the terminating zero: FoMT-US keeps at most 22 encoded
 * bytes, FoMT-JP 20, MFoMT-US 28, MFoMT-JP 20. This is byte truncation, not
 * character-aware truncation; it can split a multibyte character in modified
 * text. RIFF strings themselves are not limited to this substitution length.
 * Calendar, clock, tool-name, and invitation scripts pass RIFF text symbols
 * here and later expand them through {Var1}-{Var4}. FOMT Studio's `Check_Date`
 * label is a type-recovery error: it renders the second text reference as an
 * integer table index. This callable is slot 0x03A in FoMT and 0x03B in MFoMT.
 *
 * 将字符串写入编号文本替换槽。
 * 参数：variable_index 必须是 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4，原生 setter
 * 不做越界检查。text 会复制到该槽，在终止零字节前的编码字节上限分别为：
 * FoMT-US 22、FoMT-JP 20、MFoMT-US 28、MFoMT-JP 20。这里按字节截断，
 * 不识别字符边界，改写文本时可能截断多字节字符。RIFF 字符串本身不受这个
 * 替换槽长度上限约束。
 * 日历、时钟、工具名称及邀请脚本会在此传入 RIFF 文本符号，随后通过
 * {Var1}-{Var4} 展开。FOMT Studio 的 `Check_Date` 标签源于类型恢复错误：它把
 * 第二个文本引用显示成了整数表索引。本 callable 在 FoMT 为 0x03A，在 MFoMT
 * 为 0x03B。
 */
void SetTextVariableString(MaryTextVariableSlot variable_index, const char *text);

/* Returns the engine PRNG's nonnegative 15-bit sample.
 * Parameters: none.
 * Return value: an integer from 0 through 32767, without range reduction.
 *
 * 返回引擎伪随机数发生器的非负 15 位样本。
 * 参数：无。
 * 返回值：0 至 32767 的整数，不执行区间缩放。
 */
int RandomU15(void);

/*
 * Generates a random integer in the inclusive range [min_value, max_value].
 * Parameters: min_value is the inclusive lower bound; max_value is the
 * inclusive upper bound. Use min_value <= max_value for a meaningful range;
 * an inverted range skips random generation and pushes zero (assuming a
 * valid VM stack). This fallback is not a sample from the requested range.
 * Implementation: min_value + native_rand() % (max_value - min_value + 1).
 * Unlike RandomU15(), this uses the native 31-bit sample without a 15-bit
 * mask. The width must fit a positive signed 32-bit integer. Direct modulo
 * reduction introduces bias when the width does not divide 2147483648.
 * Return value: a generated integer inside the requested range.
 *
 * 生成闭区间 [min_value, max_value] 内的随机整数。
 * 参数：min_value 为包含在内的下界；max_value 为包含在内的上界。原生处理
 * 函数在 min_value <= max_value 时执行区间采样；反向区间会跳过随机数
 * 生成并压入零（前提是 VM 栈有效）。此回退值不是请求区间内的随机样本。
 * 实现为 min_value + native_rand() % (max_value - min_value + 1)。与
 * RandomU15() 不同，此处直接使用原生 31 位样本，不施加 15 位掩码。区间
 * 宽度必须能用正的 32 位有符号整数表示；宽度不能整除 2147483648 时，
 * 直接取余会引入分布偏差。
 * 返回值：请求区间内生成的整数。
 */
int RandomIntInclusive(int min_value, int max_value);

/*
 * Reads a numbered game-state variable, following the VarGet convention used
 * by decompilation projects such as pokeemerald.
 * Parameter: var_id is VAR_* or the exact numeric variable ID.
 * Return value: the current integer value of the selected variable.
 *
 * 读取编号游戏状态变量；命名遵循 pokeemerald 等反编译工程的 VarGet 约定。
 * 参数：var_id 为 VAR_* 或精确数字变量 ID。
 * 返回值：所选变量当前的整数值。
 */
int VarGet(MaryVarId var_id);

/*
 * Writes a numbered game-state variable.
 * Parameters: var_id is VAR_* or the exact numeric variable ID; value is the
 * requested integer value. Return value: none; use VarGet to observe the result.
 * Readability does not imply writeability: IDs below 28 have no native
 * storage setter in any target. In particular, writing VAR_PLAYER_FATIGUE
 * neither changes fatigue nor dispatches a variable-change notification.
 * Use ChangePlayerStaminaAndFatigue for relative stamina/fatigue adjustment,
 * observing its mode guard and fatigue scaling.
 * The selected setter may normalize or truncate value; it is not an arbitrary
 * integer storage cell.
 * Child age/birthday and the pregnancy-day counter are also read-only through
 * this callable. The child walking state is set-only: zero cannot clear it.
 * VAR_WEDDING_SEASON and VAR_WEDDING_DAY are read-only through VarSet in all
 * four targets: their entries skip storage. Marriage progression updates the
 * packed wedding date through a separate native path.
 * VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID also has no storage setter or
 * change notification in any target; VarSet cannot select the submitted dish.
 * Some event-state IDs also run post-write processing when their stored value
 * changes: 1 registers the ID in a deduplicated list
 * of at most three entries, while 0 and 2 remove it without reordering the
 * remaining entries. Registration can also invoke the same context-object
 * cleanup as ResetTalkUi (physical callable slot 0x01E in all four targets).
 * These effects are ID-dependent, not a universal rule for every VAR_*.
 *
 * 写入编号游戏状态变量。
 * 参数：var_id 为 VAR_* 或精确数字变量 ID；value 为写入的整数。
 * 返回值：无；应通过 VarGet 读取操作后的实际状态。
 * 可读取不代表可写入：四个目标版本的编号 28 以下均没有原生存储写入分支。
 * 特别是写入 VAR_PLAYER_FATIGUE 既不会改变疲劳，也不会分派变量变化通知。
 * 相对调整体力／疲劳应使用 ChangePlayerStaminaAndFatigue，并注意其模式门控
 * 和疲劳数值尺度。
 * 具体写入分支可能将 value 归一化或截断，并非任意整数存储单元。
 * 孩子年龄／生日及孕期天数也不能通过此函数写入。孩子学步状态只能置位，
 * 写入零不能将其清除。
 * 四版的 VAR_WEDDING_SEASON 和 VAR_WEDDING_DAY 均不能通过 VarSet 写入：
 * 对应入口跳过存储操作，婚姻进度通过另一条原生路径更新打包的婚礼日期。
 * 四版的 VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID 也没有存储写入或变化通知，
 * 不能通过 VarSet 选择提交的料理。
 * 部分事件状态 ID 在实际存储值变化后还会执行后处理：1 将 ID 登记到最多三个成员的
 * 去重列表，0 或 2 则移除该 ID，保留其余成员顺序。登记还可能调用与
 * ResetTalkUi（四版物理 callable 槽 0x01E）相同的上下文对象清理。
 * 这些副作用取决于 ID，并非所有 VAR_* 的统一规则。
 */
void VarSet(MaryVarId var_id, int value);

/*
 * Tests whether the player is holding nothing.
 * Parameters: none.
 * Return value: nonzero when no object is held; zero otherwise.
 *
 * 判断玩家当前是否没有手持物。
 * 参数：无。
 * 返回值：未手持物品时为非零，否则为零。
 */
MaryBool IsPlayerHoldingNothing(void);

/*
 * Returns the category of the player's held object.
 * Parameters: none.
 * Return value: one of the six object HELD_ITEM_KIND_* values, or
 * HELD_ITEM_KIND_NONE when no object is held.
 * Use this result before interpreting a category-specific held-object ID.
 *
 * 返回玩家当前手持物的类别。
 * 参数：无。
 * 返回值：六种对象 HELD_ITEM_KIND_* 值之一；未手持对象时返回
 * HELD_ITEM_KIND_NONE。
 * 应先依据此结果，再解释具体类别的手持物 ID。
 */
MaryHeldItemKind GetPlayerHeldItemKind(void);

/*
 * Tests whether the player's held object is gift-wrapped.
 * Parameters: none.
 * Return value: nonzero when wrapped; zero otherwise.
 *
 * 判断玩家当前手持物是否已包装。
 * 参数：无。
 * 返回值：已包装时为非零，否则为零。
 */
MaryBool IsPlayerHeldItemWrapped(void);

/*
 * Returns the food ID of the held object.
 * Parameters: none.
 * Return value: ITEM_FOOD_* or the original food ID; ITEM_FOOD_NOT_PRESENT when the held
 * object is absent or is not food. The FoMT-US native wrapper checks both the
 * empty state and held category before extracting Food::GetId; either failed
 * check returns -1 to the VM.
 *
 * 返回当前手持食品的 ID。
 * 参数：无。
 * 返回值：ITEM_FOOD_* 或原始食品 ID；当前对象不存在或不是食品时为
 * ITEM_FOOD_NOT_PRESENT。FoMT-US 原生包装器会先检查空状态和手持类别，再读取
 * Food::GetId；任一检查失败都会向 VM 返回 -1。
 */
MaryItemFoodId GetPlayerHeldFoodId(void);

/*
 * Returns the article ID of the held object.
 * Parameters: none.
 * Return value: ITEM_ARTICLE_* or the original article ID; ITEM_ARTICLE_NOT_PRESENT when
 * the held object is absent or is not an article. The FoMT-US wrapper performs
 * both checks before Article::GetId and returns -1 on either failure.
 *
 * 返回当前手持物品的 ID。
 * 参数：无。
 * 返回值：ITEM_ARTICLE_* 或原始物品 ID；当前对象不存在或不是物品时为
 * ITEM_ARTICLE_NOT_PRESENT。FoMT-US 包装器在读取 Article::GetId 前执行两项检查，
 * 任一失败都会返回 -1。
 */
MaryItemArticleId GetPlayerHeldArticleId(void);

/*
 * Returns the chicken-record slot represented by the held object.
 * Parameters: none.
 * Return value: ANIMAL_CHICKEN_SLOT_1 through ANIMAL_CHICKEN_SLOT_8 for a held chicken, or
 * ANIMAL_CHICKEN_SLOT_NONE when the held object is not a valid chicken record.
 *
 * 返回当前手持鸡所对应的鸡记录槽位。
 * 参数：无。
 * 返回值：手持有效鸡记录时为 ANIMAL_CHICKEN_SLOT_1 至 ANIMAL_CHICKEN_SLOT_8；当前手持物
 * 不是有效鸡记录时为 ANIMAL_CHICKEN_SLOT_NONE。
 */
MaryAnimalChickenSlotIndex GetPlayerHeldChickenId(void);

/*
 * Runs the player's normal context-sensitive action for the current held
 * object and waits for that action to complete. Scripts use it when handing
 * a gift to an NPC or otherwise using the held object through the ordinary
 * player action path. The engine chooses the animation from the held-object
 * kind; this is not merely an animation command.
 * Parameters: none.
 *
 * 对当前手持物执行玩家通常的、依上下文决定的使用动作，并等待动作完成。
 * 脚本在向 NPC 交付礼物，或通过普通玩家动作路径使用手持物时调用它。
 * 引擎会依据手持物类别选择动作动画，因此它不只是单纯的动画指令。
 * 参数：无。
 */
void UsePlayerHeldItem(void);

/*
 * Removes the current held object immediately and restores the player's held
 * object state, without running the normal held-object action animation.
 * Scripted eating and drinking sequences call this and then control their own
 * sound and player animation. Article 0x35 also receives its engine-defined
 * special side effect before removal.
 * Parameters: none.
 *
 * 立即移除当前手持物并恢复玩家的手持状态，不执行通常的手持物使用动画。
 * 脚本化的进食、饮用流程会先调用本函数，再自行控制音效和玩家动画。
 * 物品 0x35 在移除前还会触发引擎定义的特殊副作用。
 * 参数：无。
 */
void ClearPlayerHeldItem(void);

/*
 * Constructs a food object, equips it as the player's unwrapped held object,
 * and starts the engine's held-object transition (player action state 0x19).
 * The native wrapper converts the argument through the target's compact Food
 * representation; it does not provide a script-level range error.
 * Parameter: food_id is ITEM_FOOD_* or the exact original valid food ID.
 *
 * 构造食品对象，将其装备为玩家的未包装手持物，并启动引擎的手持物切换流程
 * （玩家动作状态 0x19）。原生包装函数会把参数转换为目标版本的紧凑 Food
 * 结构，不会提供脚本层的越界错误。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始有效食品 ID。
 */
void SetPlayerHeldFood(MaryItemFoodId food_id);

/*
 * Constructs an article object, equips it as the player's unwrapped held
 * object, and starts the same held-object transition as SetPlayerHeldFood.
 * The native wrapper stores the article selector through its compact byte
 * representation; it does not provide a script-level range error.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original valid article ID.
 *
 * 构造物品对象，将其装备为玩家的未包装手持物，并启动与
 * SetPlayerHeldFood 相同的手持物切换流程。原生包装函数通过紧凑字节结构保存
 * 物品选择值，不会提供脚本层的越界错误。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始有效物品 ID。
 */
void SetPlayerHeldArticle(MaryItemArticleId article_id);

/*
 * Performs the same food construction and held-object transition as
 * SetPlayerHeldFood, with the native wrapped flag set to one.
 * Parameter: food_id is ITEM_FOOD_* or the exact original valid food ID.
 *
 * 执行与 SetPlayerHeldFood 相同的食品构造和手持物切换流程，但把原生包装标志
 * 设为 1。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始有效食品 ID。
 */
void SetPlayerHeldWrappedFood(MaryItemFoodId food_id);

/*
 * Performs the same article construction and held-object transition as
 * SetPlayerHeldArticle, with the native wrapped flag set to one.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original valid article ID.
 *
 * 执行与 SetPlayerHeldArticle 相同的物品构造和手持物切换流程，但把原生包装
 * 标志设为 1。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始有效物品 ID。
 */
void SetPlayerHeldWrappedArticle(MaryItemArticleId article_id);

/*
 * Tests whether the player is holding an Article whose native
 * Article::CanBeDiscarded policy permits disposal. The wrapper first rejects
 * an empty hand and every held-item kind other than HELD_ITEM_KIND_ARTICLE,
 * then passes the current article ID to the native policy. It does not discard
 * or otherwise mutate the held item.
 * Parameters: none.
 * Return value: TRUE only when all three gates pass; FALSE otherwise.
 *
 * 判断玩家是否手持 Article，且该物品通过原生
 * Article::CanBeDiscarded 丢弃策略。包装函数会先拒绝空手及
 * HELD_ITEM_KIND_ARTICLE 以外的所有手持类型，再把当前物品 ID
 * 交给原生策略。它不会真正丢弃或修改手持物。
 * 参数：无。
 * 返回值：三层条件全部通过时为 TRUE，否则为 FALSE。
 */
MaryBool CanDiscardPlayerHeldArticle(void);

/*
 * Asks the shipment-box interaction handler to accept the currently held
 * object. On acceptance, the native wrapper changes the player to action
 * state 0x19; the calling script then starts the shipment-box deposit
 * animation and sound. It does not itself run that box animation.
 * Parameters: none.
 * Return value: TRUE when the shipment-box handler accepted the held object
 * and the player action was started; FALSE when it rejected the object.
 *
 * 请求出货箱交互处理器接收当前手持物。接收成功时，原生包装函数会把玩家切换
 * 到动作状态 0x19；调用脚本随后再启动出货箱的投入动画和音效。本函数自身并不
 * 播放该箱体动画。
 * 参数：无。
 * 返回值：出货箱处理器接收手持物并启动玩家动作时为 TRUE；拒绝时为 FALSE。
 */
MaryBool TryShipPlayerHeldItem(void);

/*
 * Throws the current held object in the direction the player is facing and
 * waits for the normal throw action to complete. The engine supplies the
 * held-kind-specific animation and forward motion.
 * Parameters: none.
 *
 * 朝玩家当前面对的方向投出手持物，并等待通常的投掷动作完成。
 * 引擎会依据手持物类别选择动画并设置向前运动。
 * 参数：无。
 */
void ThrowPlayerHeldItem(void);

/*
 * Returns the tool ID of the held object.
 * Parameters: none.
 * Return value: ITEM_TOOL_* or the original tool ID; ITEM_TOOL_NOT_PRESENT when the held
 * tool stack is empty. The FoMT-US native wrapper initializes the result to
 * -1 and replaces it only after ToolStack::IsEmpty returns false; the retail
 * scripts use that sentinel before giving the fishing rod. MFoMT-US/JP call
 * sites use the same contract.
 *
 * 返回当前手持工具的 ID。
 * 参数：无。
 * 返回值：ITEM_TOOL_* 或原始工具 ID；手持工具堆为空时为 ITEM_TOOL_NOT_PRESENT。
 * FoMT-US 原生包装器先把结果初始化为 -1，仅在 ToolStack::IsEmpty 为假时才
 * 写入工具 ID；原版脚本会在赠送钓竿前使用该哨兵。MFoMT-US/JP 调用点使用
 * 相同约定。
 */
MaryItemToolId GetPlayerHeldToolId(void);

/*
 * Returns the number of copies in the player's held tool stack. This is an
 * inventory stack count, not a tool charge, upgrade level, or durability.
 * Parameters: none.
 * Return value: 1-99 for a nonempty stack, or
 * ITEM_TOOL_STACK_NOT_PRESENT when no valid held tool stack exists.
 *
 * 返回玩家当前手持工具堆中的数量。该字段是背包堆叠数量，不是工具蓄力、
 * 升级等级或耐久度。
 * 参数：无。
 * 返回值：非空堆为 1-99；不存在有效手持工具堆时为
 * ITEM_TOOL_STACK_NOT_PRESENT。
 */
MaryItemToolStackCount GetPlayerHeldToolStackCount(void);

/*
 * Sets the player's held object to a tool.
 * Parameters: tool_id is ITEM_TOOL_* or the exact original tool ID; stack_count is
 * the inventory quantity stored with that tool. The native ToolStack
 * constructor converts zero to one and clamps values greater than 99 to 99.
 * Its comparison is unsigned, so negative VM integers also become 99; scripts
 * should use the declared 1-99 domain rather than relying on this fallback.
 * Applying the stack uses the player's normal held-tool transition path.
 *
 * 将玩家手持物设置为工具。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID；stack_count 为与工具一同保存的
 * 背包堆叠数量。原生 ToolStack 构造函数会把 0 转成 1，并把大于 99 的值
 * 限制为 99。其比较采用无符号数，因此负的 VM 整数也会变成 99；脚本应使用
 * 声明的 1-99 取值域，不应依赖该回退行为。应用工具堆时会进入玩家通常的
 * 手持工具切换路径。
 */
void SetPlayerHeldTool(MaryItemToolId tool_id, MaryItemToolRequestedStackCount stack_count);

/*
 * Constructs an empty ToolStack and applies it through the same player
 * held-tool transition path used by SetPlayerHeldTool.
 * Parameters: none.
 *
 * 构造空 ToolStack，并通过与 SetPlayerHeldTool 相同的玩家手持工具切换路径
 * 应用它。
 * 参数：无。
 */
void ClearPlayerHeldTool(void);

/*
 * Finds a food in the rucksack.
 * Parameter: food_id is ITEM_FOOD_* or the exact original food ID.
 * Return value: the zero-based item slot, or RUCKSACK_SLOT_NOT_FOUND when the
 * requested food is absent.
 *
 * 在背包中查找食品。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始食品 ID。
 * 返回值：从零开始的物品槽位；没有该食品时返回 RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex FindFoodInRucksack(MaryItemFoodId food_id);

/*
 * Finds an article in the rucksack.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original article ID.
 * Return value: the zero-based item slot, or RUCKSACK_SLOT_NOT_FOUND when the
 * requested article is absent.
 *
 * 在背包中查找物品。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始物品 ID。
 * 返回值：从零开始的物品槽位；没有该物品时返回 RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex FindArticleInRucksack(MaryItemArticleId article_id);

/*
 * Clears one rucksack item slot.
 * Parameter: slot_id is the zero-based rucksack item-slot index and accepts a
 * MaryRucksackSlotIndex symbol or the identical raw integer. The native path
 * indexes the currently allocated item vector directly and performs no bounds
 * check; RUCKSACK_SLOT_NOT_FOUND and locked/out-of-range slots are invalid.
 *
 * 清空一个背包物品槽。
 * 参数：slot_id 为从 0 开始的背包物品槽序号，可使用 MaryRucksackSlotIndex
 * 符号或数值相同的原始整数。原生路径会直接索引当前分配的物品向量，不执行
 * 边界检查；RUCKSACK_SLOT_NOT_FOUND、尚未解锁及越界槽位均不可传入。
 */
void ClearRucksackItemSlot(MaryRucksackSlotIndex slot_id);

/*
 * Finds the first free rucksack tool slot.
 * Parameters: none.
 * Return value: the zero-based tool slot, or RUCKSACK_SLOT_NOT_FOUND when all
 * currently unlocked tool slots are occupied.
 *
 * 查找第一个空闲背包工具槽。
 * 参数：无。
 * 返回值：从零开始的农具槽位；当前已解锁农具槽全部占用时返回
 * RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex GetFirstFreeRucksackToolSlot(void);

/*
 * Finds the first free rucksack item slot.
 * Parameters: none.
 * Return value: the zero-based item slot, or RUCKSACK_SLOT_NOT_FOUND when all
 * currently unlocked item slots are occupied.
 *
 * 查找第一个空闲背包物品槽。
 * 参数：无。
 * 返回值：从零开始的物品槽位；当前已解锁物品槽全部占用时返回
 * RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex GetFirstFreeRucksackItemSlot(void);

/*
 * Attempts to add an article stack to the rucksack.
 * Parameters: article_id is ITEM_ARTICLE_* or the exact original article ID;
 * requested_count is a nonnegative number of copies to add. Each article
 * occupies one item slot. The native routine treats the quantity as u32 and
 * does not reject negative Mary-C values.
 * Return value: the number of copies that could not be added. Zero means the
 * full request fit in the currently unlocked item slots.
 *
 * 尝试向背包加入一组物品。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始物品 ID；requested_count 为请求
 * 加入的非负份数。每份物品占用一个物品槽。原生例程把数量视为 u32，不会
 * 拒绝 Mary-C 负值。
 * 返回值：未能加入背包的剩余份数；返回 0 表示请求数量全部装入当前已解锁
 * 的物品槽。
 */
MaryItemUnaddedCount AddArticleToRucksack(
    MaryItemArticleId article_id,
    MaryRequestedInventoryCount requested_count
);

/*
 * Attempts to add a food stack to the rucksack.
 * Parameters: food_id is ITEM_FOOD_* or the exact original food ID;
 * requested_count is a nonnegative number of copies to add. Each food occupies
 * one item slot. The native routine treats the quantity as u32 and does not
 * reject negative Mary-C values.
 * Return value: the number of copies that could not be added. Zero means the
 * full request fit in the currently unlocked item slots.
 *
 * 尝试向背包加入一组食品。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始食品 ID；requested_count 为请求加入的
 * 非负份数。每份食品占用一个物品槽。原生例程把数量视为 u32，不会拒绝
 * Mary-C 负值。
 * 返回值：未能加入背包的剩余份数；返回 0 表示请求数量全部装入当前已解锁
 * 的物品槽。
 */
MaryItemUnaddedCount AddFoodToRucksack(
    MaryItemFoodId food_id,
    MaryRequestedInventoryCount requested_count
);

/*
 * Attempts to add a quantity of one tool ID to the rucksack. Existing stacks
 * of the same tool are filled first and each tool stack holds at most 99.
 * Parameters: tool_id is ITEM_TOOL_* or the exact original tool ID;
 * requested_count is the nonnegative inventory quantity to add. The native
 * routine treats it as u32 and does not reject negative Mary-C values.
 * Return value: the quantity that could not be added. Zero means the full
 * request fit in the currently unlocked tool slots.
 *
 * 尝试向背包加入指定数量的同一种工具。引擎会先填充已有的同 ID 工具堆，
 * 每个工具堆最多保存 99 个。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID；requested_count 为请求加入的
 * 非负背包数量。原生例程把它视为 u32，不会拒绝 Mary-C 负值。
 * 返回值：未能加入背包的剩余数量；返回 0 表示请求数量全部装入当前已解锁
 * 的工具槽。
 */
MaryItemUnaddedCount AddToolToRucksack(
    MaryItemToolId tool_id,
    MaryRequestedInventoryCount requested_count
);

/*
 * Makes the player actor visibly hold or present a tool.
 * Parameter: tool_id is ITEM_TOOL_* or the exact original tool ID. This is used
 * when receiving tools, presenting the Blue Feather, showing a blessed tool,
 * and receiving the Goddess, Kappa, or Truth Gem.
 * The native routine invokes the player's presentation callback and changes
 * the event/player action state to 0x19; it does not add the tool to storage.
 *
 * 让玩家角色以可见方式手持或展示工具。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID。获得工具、展示蓝色羽毛、
 * 展示解除诅咒后的工具，以及取得女神、河童或真实之玉时都会调用它。
 * 原生例程会调用玩家的展示回调，并把事件／玩家动作状态切换为 0x19；它不会
 * 把工具加入任何存储容器。
 */
void ShowPlayerHoldingTool(MaryItemToolId tool_id);

/*
 * Applies signed changes to player stamina and fatigue.
 * Parameters: stamina_delta changes stamina; fatigue_delta changes fatigue.
 * Positive values increase and negative values decrease the corresponding
 * stat. All four vanilla script sets use the same contract for sleep recovery,
 * bathroom and fireplace effects, collapse treatment, and Doctor/Kai events.
 * This callable is physical slot 0x05B in FoMT and 0x05C in MFoMT because
 * MFoMT inserts ShowPlayerHoldingTool immediately before it.
 * fatigue_delta is an adjustment input, not a raw stored-byte delta. On the
 * normal adjustment path, positive fatigue is doubled without the Mystic
 * Berry and left undoubled with it; negative fatigue is doubled in either
 * case. The fatigue helper uses an internal upper limit of 200. Do not treat
 * this internal scale as a displayed percentage or use extreme integers.
 * VarGet(VAR_PLAYER_FATIGUE) returns floor(internal fatigue / 2), not the
 * raw stored value; odd internal changes can be invisible in a single read.
 * While EnableScriptedNpcControl's global mode is active, the player entity
 * returns before applying either adjustment.
 *
 * 对玩家体力与疲劳应用有符号变化量。
 * 参数：stamina_delta 改变体力；fatigue_delta 改变疲劳。
 * 正数提高对应数值，负数降低对应数值。四套原版脚本在睡眠恢复、浴室与壁炉
 * 效果、昏倒治疗以及 Doctor/Kai 事件中都使用相同参数约定。该 callable 在
 * FoMT 的物理槽为 0x05B；MFoMT 在它之前插入了 ShowPlayerHoldingTool，故其
 * 物理槽顺延为 0x05C。
 * fatigue_delta 是调整输入，不是原始存储字节的变化量。正常调整路径中，
 * 正疲劳输入在未取得神秘果实时翻倍、取得后不翻倍；负疲劳输入则始终翻倍。
 * 疲劳辅助函数的内部上限为 200，不能将此内部尺度直接当成显示百分比，
 * 也不应传入极端整数。
 * VarGet(VAR_PLAYER_FATIGUE) 返回内部疲劳值除以 2 后向下取整的结果，而非
 * 原始存储值；内部增加奇数时，单次读取可能看不出变化。
 * EnableScriptedNpcControl 启用的全局模式生效期间，玩家实体会在应用这两项
 * 调整之前直接返回。
 */
void ChangePlayerStaminaAndFatigue(
    MaryStaminaDelta stamina_delta,
    MaryFatigueDelta fatigue_delta
);

/*
 * Tests whether the player is currently holding a tool.
 * Parameters: none.
 * Return value: nonzero when a tool is held; zero otherwise.
 *
 * 判断玩家当前是否手持工具。
 * 参数：无。
 * 返回值：手持工具时为非零，否则为零。
 */
MaryBool IsPlayerHoldingTool(void);

/*
 * Tests whether the specified tool is currently held, present in the
 * rucksack, or stored in the tool chest.
 * Parameter: tool_id is ITEM_TOOL_* or the exact original tool ID.
 * Return value: nonzero when owned; zero otherwise.
 *
 * 判断指定工具是否正被手持、位于背包或存放在工具箱中。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsTool(MaryItemToolId tool_id);

/* Tests whether the player owns a specific food in the held-item slot,
 * rucksack, or refrigerator.
 * Parameter: food_id is ITEM_FOOD_* or the exact
 * original food ID. Return value: nonzero when owned; zero otherwise.
 *
 * 判断手持物槽、背包或冰箱中是否拥有指定食品。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始食品 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsFood(MaryItemFoodId food_id);

/*
 * Tests whether the specified article is currently held, present in the
 * rucksack, or stored on the shelf.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original article ID.
 * Return value: nonzero when owned; zero otherwise.
 *
 * 判断指定物品是否正被手持、位于背包或存放在置物架中。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始物品 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsArticle(MaryItemArticleId article_id);

/*
 * Removes every copy of an article from the current held-object slot, all
 * currently unlocked rucksack item slots, and every shelf slot. Wrapped and
 * unwrapped copies are both matched by article ID. Other containers and item
 * categories are not affected.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original valid article ID.
 *
 * 从当前手持物槽、所有已解锁背包物品槽以及全部置物架槽中移除指定物品的每份
 * 副本。已包装和未包装副本都会按物品 ID 匹配；其他容器和物品类别不受影响。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始有效物品 ID。
 */
void RemoveAllOwnedArticles(MaryItemArticleId article_id);

/*
 * Gives the player one Power Berry and runs the engine's acquisition action.
 * Parameters: none. This updates the Farmer power-berry state itself; event
 * flags that prevent an individual berry from being collected twice remain
 * the responsibility of the calling script.
 * The acquisition routine updates the count before setting up its animation.
 * The counter helper increments only when the existing count is below 10;
 * a count of 10 or greater is left unchanged, not reset to 10.
 * A successful increment also adds 10 current stamina, capped at the newly
 * computed maximum; it does not fully heal the player. At count >= 10 the
 * helper skips both the increment and stamina recovery.
 *
 * 给予玩家一枚力量果实，并执行引擎的取得动作。
 * 参数：无。
 * 本函数会直接更新 Farmer 的力量果实状态；用于防止某一枚果实被重复取得的
 * 事件标志，仍由调用脚本负责设置。
 * 取得处理函数先更新计数，再设置动画。计数辅助函数只在原计数小于 10 时
 * 增加；已有计数大于等于 10 时保持原值，不会强制修正为 10。
 * 成功增加计数时也增加当前体力 10 点，以新的体力上限为限，并非回满体力。
 * 计数已大于等于 10 时，计数增加和体力恢复都会跳过。
 */
void ObtainPowerBerry(void);

/*
 * Gives the player the Mystic Berry obtained from Kappa after offering ten
 * cucumbers, and runs the corresponding engine acquisition action.
 * Parameters: none. The calling event separately records that this unique
 * reward has been claimed.
 * Its Farmer-state helper sets a single flag; it does not increment
 * the Power Berry count or restore stamina. Repeating it leaves that flag set
 * rather than stacking another numerical bonus.
 * Read the acquired state with VarGet(VAR_HAS_MYSTIC_BERRY). This is the
 * Farmer flag, separate from the Kappa reward event's progression state;
 * VarSet cannot write this read-only variable ID.
 *
 * 给予玩家向 Kappa 供奉十根黄瓜后获得的神秘果实，并执行对应的引擎取得动作。
 * 参数：无。该唯一奖励是否已经领取，仍由调用事件另行记录。
 * 其 Farmer 状态辅助函数仅设置一个标志位，不增加力量果实计数，也不恢复体力。
 * 重复调用只是保持该标志已设置，并不会叠加新的数值奖励。
 * 用 VarGet(VAR_HAS_MYSTIC_BERRY) 读取取得状态；这是 Farmer 的标志，
 * 与河童奖励事件的进度状态不同。该变量编号只读，不能通过 VarSet 写入。
 */
void ObtainMysticBerry(void);

/*
 * Tests whether the player is currently riding the horse.
 * Parameters: none.
 * Return value: nonzero while mounted; zero otherwise.
 * Scripts use this check to block indoor transitions and interactions that are
 * unavailable while riding.
 *
 * 判断玩家当前是否正在骑马。
 * 参数：无。
 * 返回值：骑乘时为非零，否则为零。
 * 脚本通过该检查阻止骑马时进入室内，或执行骑乘状态下不可用的交互。
 */
MaryBool IsPlayerRidingHorse(void);

/*
 * Enters the player's persistent hot-spring bathing actor state after the
 * scripted jump into the spring.
 * Parameters: none. The matching exit callable
 * must be used before normal movement resumes.
 *
 * 在脚本控制玩家跳入温泉后，进入持续的温泉入浴角色状态。
 * 参数：无。恢复常规
 * 移动前必须调用与之配对的退出函数。
 */
void EnterHotSpringBathingState(void);

/*
 * Leaves the player's hot-spring bathing actor state. The hot-spring exit and
 * forced recovery scripts also call it defensively before assigning normal
 * player animation and movement. Across each of the four vanilla script sets,
 * these are its only two call sites and both invoke it with no operands. The
 * `Kill_NPC(0)` rendering in FOMT Studio comes from that tool's incorrect
 * one-argument declaration; the original bytecode does not supply an entity
 * ID, and the callable is paired with EnterHotSpringBathingState.
 * Parameters: none.
 *
 * 退出玩家的温泉入浴角色状态。温泉出口及强制恢复剧情也会在指定常规玩家动画
 * 与移动之前防御性调用本函数。四套原版脚本中，每套都只有这两个调用点，且都
 * 不提供操作数。FOMT Studio 输出的 `Kill_NPC(0)` 源于其错误的一参数声明；
 * 原始字节码并未提供实体 ID，本 callable 实际与 EnterHotSpringBathingState
 * 成对使用。
 * 参数：无。
 */
void ExitHotSpringBathingState(void);

/*
 * Preserves the player's current map and return-position context so the daily
 * transition can resume there after advancing to the next day.
 * Parameters:
 * none. Scripts call this before overnight transitions that should not send
 * the player back to the farmhouse bed.
 *
 * 保存玩家当前地图及返回位置上下文，使每日转场推进到次日后仍能从该处继续。
 * 参数：无。需要避免把玩家送回农舍床位的过夜转场会先调用本函数。
 */
void PreservePlayerLocationForNextDay(void);

/*
 * Clears the preserved overnight return location and resets its map to
 * MAP_NONE, causing the normal next-day spawn path to be used.
 * Parameters:
 * none.
 *
 * 清除已保存的过夜返回位置，并将其地图重置为 MAP_NONE，使次日使用常规出生
 * 位置流程。
 * 参数：无。
 */
void ClearPreservedPlayerLocation(void);

/*
 * Returns the map stored by PreservePlayerLocationForNextDay, or MAP_NONE
 * when no return location is active.
 * Parameters: none. Return value: the
 * preserved MaryMapId; it is not necessarily the currently displayed map.
 *
 * 返回 PreservePlayerLocationForNextDay 保存的地图；没有有效返回位置时返回
 * MAP_NONE。
 * 参数：无。
 * 返回值：保存的 MaryMapId；它不一定是当前显示的地图。
 */
MaryMapId GetPreservedPlayerMapId(void);

/*
 * Makes the player eat one randomly selected cooked meal from the saved meal
 * list. When that list has no usable entry, the engine falls back to a Rice
 * Ball or Bread. The selected Food's stamina and fatigue effects and eating
 * animation are applied.
 * Parameters: none.
 *
 * 让玩家从已保存的料理列表中随机吃下一份料理。列表中没有可用项目时，引擎
 * 会退回到饭团或面包。所选 Food 的体力、疲劳效果及进食动画都会生效。
 * 参数：无。
 */
void EatRandomMeal(void);

/*
 * Puts the player actor into the neutral scripted-animation state used before
 * a cutscene directly assigns position, facing, or animation. It disables the
 * actor's normal animation selection and selects the neutral animation.
 * Parameters: none.
 *
 * 将玩家角色置于脚本动画使用的中立状态，供剧情直接指定位置、朝向或动画。
 * 本函数会停用角色的常规动画选择并切换到中立动画。
 * 参数：无。
 */
void PreparePlayerForScriptedAnimation(void);

/*
 * Leaves the scripted-animation state and rebuilds the player's normal actor
 * state and animation from the current map, held-item, tool, and movement
 * context. The callable waits for the actor transition to complete.
 * Parameters: none.
 *
 * 退出脚本动画状态，并根据当前地图、手持物品、农具及移动上下文重建玩家的
 * 常规角色状态和动画。本调用会等待角色转场完成。
 * 参数：无。
 */
void RestorePlayerAfterScriptedAnimation(void);

/*
 * Returns the kind of the item captured by the current item-presentation
 * event. The Harvest Goddess offering scripts prove FOOD and ARTICLE results.
 * Parameters: none.
 * Return value: a HELD_ITEM_KIND_* value describing the captured item.
 *
 * 返回当前“提交物品”事件所捕获物品的类别。女神供品脚本已证明会返回 FOOD
 * 与 ARTICLE。
 * 参数：无。
 * 返回值：描述所捕获物品类别的 HELD_ITEM_KIND_* 值。
 */
MaryHeldItemKind GetPresentedItemKind(void);

/*
 * Returns the captured presentation ID. The native player actor stores this
 * cache only for FOOD and ARTICLE presentations: it is a food ID for FOOD and
 * an article ID for ARTICLE. Other presented kinds do not establish an ID
 * domain here, so callers must not infer one from HeldItem layout alone. A
 * single static return enum would therefore be misleading.
 * Parameters: none.
 * Return value: the captured raw ID; interpret it using GetPresentedItemKind().
 *
 * 返回“提交物品”流程捕获的 ID。原生玩家角色只在 FOOD 与 ARTICLE 提交流程中
 * 写入这项缓存：FOOD 对应食品 ID，ARTICLE 对应物品 ID。其他提交类别不会在
 * 此处建立 ID 取值域，不能仅凭 HeldItem 内部布局推导其含义。因此不能安全地
 * 声明为单一静态枚举类型。
 * 参数：无。
 * 返回值：捕获的原始 ID；必须结合 GetPresentedItemKind() 解释。
 */
int GetPresentedItemId(void);

/*
 * Returns whether the item captured by GetPresentedItemKind() and
 * GetPresentedItemId() has the gift-wrap bonus. The offering scripts apply the
 * documented 25 percent relationship bonus when this value is nonzero.
 * Parameters: none.
 * Return value: TRUE when the captured item is wrapped; FALSE otherwise.
 *
 * 返回 GetPresentedItemKind()/GetPresentedItemId() 捕获的物品是否带有礼物
 * 包装加成。供品脚本在本值非零时会应用 25% 的关系值加成。
 * 参数：无。
 * 返回值：捕获物品已包装时为 TRUE，否则为 FALSE。
 */
MaryBool IsPresentedItemGiftWrapped(void);

/* Tests whether the held-tool slot, rucksack, or tool chest can accept a tool.
 * An empty held-tool slot accepts it; a nonempty held stack accepts only the
 * same tool ID while its amount is below 99.
 * Parameter: tool_id is ITEM_TOOL_* or the exact original tool ID.
 * Return value: nonzero when at least one applicable destination can accept
 * the tool; zero when all applicable destinations are full.
 *
 * 判断手持工具槽、背包或工具箱能否容纳指定工具。手持工具槽为空时可以接收；
 * 非空时只有同 ID 且数量小于 99 的工具堆可以继续接收。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID。
 * 返回值：至少一个适用位置能够容纳时为非零；所有适用位置均已满时为零。
 */
MaryBool CanReceiveTool(MaryItemToolId tool_id);

/*
 * Tests whether a food item can be received without being lost.
 * Parameter: food_id is ITEM_FOOD_* or the exact original food ID.
 * Return value: nonzero when the empty held-item slot, an unlocked rucksack
 * item slot, or the refrigerator can accept the food; zero when all
 * applicable destinations are full.
 * Shop scripts call this after the price check and before granting the item.
 *
 * 判断食品能否被玩家接收且不会丢失。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始食品 ID。
 * 返回值：空手持物槽、已解锁背包物品槽或冰箱中至少一处能够容纳该食品时为
 * 非零；所有适用存放位置均已满时为零。商店脚本会在检查价格之后、交付商品
 * 之前调用它。
 */
MaryBool CanReceiveFood(MaryItemFoodId food_id);

/* Tests whether the empty held-item slot, an unlocked rucksack item slot, or
 * the shelf can accept an article.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original article ID.
 * Return value: nonzero when at least one applicable destination can accept
 * the article; zero when all applicable destinations are full.
 *
 * 判断空手持物槽、已解锁背包物品槽或置物架能否容纳指定物品。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始物品 ID。
 * 返回值：至少一个适用位置能够容纳时为非零；所有适用位置均已满时为零。
 */
MaryBool CanReceiveArticle(MaryItemArticleId article_id);

/*
 * Gives the basket to the player.
 * Parameters: none.
 *
 * 将篮子交给玩家。
 * 参数：无。
 */
void GivePlayerBasket(void);

/*
 * Tests whether the player owns the basket.
 * Parameters: none.
 * Return value: nonzero when owned; zero otherwise.
 *
 * 判断玩家是否拥有篮子。
 * 参数：无。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerHasBasket(void);

/*
 * Advances the rucksack to its next capacity tier.
 * Parameters: none.
 *
 * 将背包提升到下一容量等级。
 * 参数：无。
 */
void UpgradeRucksack(void);

/* Returns the current rucksack capacity-upgrade level.
 * Parameters: none.
 * Return value: a RUCKSACK_UPGRADE_LEVEL_* value.
 *
 * 返回当前背包容量升级等级。
 * 参数：无。
 * 返回值：RUCKSACK_UPGRADE_LEVEL_* 值。
 */
MaryRucksackUpgradeLevel GetRucksackUpgradeLevel(void);

/*
 * Enables or disables the player actor's normal per-frame update.
 * Parameter:
 * suspended is treated as a boolean; nonzero skips the normal movement and
 * animation-selection update, while zero restores it. This does not pause the
 * script engine or dialogue system.
 *
 * 启用或停用玩家角色的常规逐帧更新。
 * 参数：suspended 按布尔值处理；非零时
 * 跳过常规移动及动画选择更新，零则恢复。它不会暂停脚本引擎或对话系统。
 */
void SetPlayerActorUpdateSuspended(MaryBool suspended);
/*
 * Reads the selected milestone bit from the local link-data record. Normal
 * event scripts set these bits when relationship or farm milestones are met;
 * the link sequence later transfers and may clear them.
 * Parameter: milestone_id is LOCAL_LINK_MILESTONE_* in the complete
 * target-specific local bit domain. Return value: nonzero when set.
 *
 * 从本地联机数据记录中读取指定里程碑位。普通事件脚本会在人物关系或农场条件
 * 达成时设置这些位；联机流程随后会传输并可能清除它们。
 * 参数：milestone_id 为目标版本完整本地位域中的 LOCAL_LINK_MILESTONE_*。
 * 返回值：该位已设置时为非零。
 */
MaryBool HasLocalLinkMilestone(MaryLocalLinkMilestoneId milestone_id);

/*
 * Reads a milestone bit from the link partner's received data. The received
 * bitset is stored separately from the local pending-milestone bitset.
 * Parameter: milestone_id is RECEIVED_LINK_MILESTONE_* in the complete
 * target-specific received bit domain. Return value: nonzero when set.
 *
 * 从联机伙伴的接收数据中读取指定里程碑位。接收位域与本地待传输里程碑位域
 * 分开存储。
 * 参数：milestone_id 为目标版本完整接收位域中的 RECEIVED_LINK_MILESTONE_*。
 * 返回值：该位已设置时为非零。
 */
MaryBool HasReceivedLinkMilestone(MaryReceivedLinkMilestoneId milestone_id);

/*
 * Sets one local pending link milestone.
 * Parameter: milestone_id is the zero-based bit index in the local link
 * record. This does not modify the separately stored data received from a
 * link partner.
 *
 * 设置一个本地待传输联机里程碑。
 * 参数：milestone_id 为本地联机记录中从零开始的位索引。本函数不会修改另行
 * 存储的联机伙伴接收数据。
 */
void SetLocalLinkMilestone(MaryLocalLinkMilestoneId milestone_id);

/*
 * Clears one local pending link milestone.
 * Parameter: milestone_id is the zero-based bit index in the local link
 * record. This does not modify the separately stored data received from a
 * link partner.
 *
 * 清除一个本地待传输联机里程碑。
 * 参数：milestone_id 为本地联机记录中从零开始的位索引。本函数不会修改另行
 * 存储的联机伙伴接收数据。
 */
void ClearLocalLinkMilestone(MaryLocalLinkMilestoneId milestone_id);

/*
 * Tests whether today is a character's birthday. The native handler obtains
 * the character's encoded birthday from gCharacterNameEntries, applies the
 * original conflict-adjustment rules for selected characters, and compares
 * its low seven date bits with the current calendar date. Gift scripts use
 * this result both to select birthday dialogue and to apply the fivefold
 * birthday relationship bonus.
 * Parameter: character_id is CHARACTER_* or the exact target-specific ID.
 * Return value: nonzero when today is that character's birthday; zero
 * otherwise.
 *
 * 判断今天是否为指定人物的生日。原生处理函数从 gCharacterNameEntries 取得编码
 * 后的生日，对部分人物应用原版的生日冲突调整规则，再将日期低七位与当前日历日期
 * 比较。礼物脚本同时用该结果选择生日台词，并应用五倍的生日关系值奖励。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确人物 ID。
 * 返回值：今天是该人物生日时为非零，否则为零。
 */
MaryBool IsCharacterBirthdayToday(MaryCharacterId character_id);

/*
 * Gets an NPC's friendship points.
 * Parameter: character_id is CHARACTER_* or the exact target-specific ID.
 * Return value: the NPC's current friendship value, or zero when the ID does
 * not resolve to an NPC.
 *
 * 获取 NPC 的友好度点数。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确 ID。
 * 返回值：NPC 当前的友好度；ID 无法解析为 NPC 时返回零。
 */
MaryNpcFriendshipValue GetNpcFriendship(MaryCharacterId character_id);

/*
 * Adds a signed amount to an NPC's friendship points.
 * Parameters: character_id selects the NPC; amount is the signed adjustment.
 * This is the modifying counterpart of GetNpcFriendship.
 * The native 32-bit sum is limited to 0-255 after addition. Avoid extreme
 * adjustments that overflow signed 32-bit arithmetic; they can wrap before
 * the limit checks. An unresolved character ID leaves NPC state unchanged.
 *
 * 对 NPC 的友好度点数增加一个有符号数值。
 * 参数：character_id 选择 NPC；amount 为有符号调整量。
 * 本函数是 GetNpcFriendship 对应的修改操作。
 * 原生先进行 32 位加法，再将结果限制到 0-255。应避免导致有符号 32 位
 * 溢出的极端调整量，因为结果可能在范围检查前回绕。人物 ID 无法解析时，
 * 不修改 NPC 状态。
 */
void AddNpcFriendship(MaryCharacterId character_id, MaryNpcFriendshipDelta amount);

/*
 * Replaces an NPC's friendship value.
 * Parameters: character_id selects the NPC; friendship is the new value.
 * This directly stores the low eight bits, unlike AddNpcFriendship's limit
 * checks: 256 becomes 0, and -1 becomes 255. An unresolved ID is ignored.
 *
 * 直接替换 NPC 的友好度数值。
 * 参数：character_id 选择 NPC；friendship 为新的友好度数值。
 * 本函数直接写入低八位，不执行 AddNpcFriendship 的范围限制：256 变成 0，
 * -1 变成 255。无法解析的人物 ID 被忽略。
 */
void SetNpcFriendship(MaryCharacterId character_id, MaryNpcFriendshipValue friendship);

/*
 * Gets the number of days since the player last spoke to an NPC.
 * Parameter: character_id selects the NPC.
 * Return value: the stored day count, or zero when the ID is invalid.
 *
 * 获取玩家上次与某 NPC 交谈后经过的天数。
 * 参数：character_id 选择 NPC。
 * 返回值：保存的天数；ID 无效时返回零。
 */
MaryDaysSinceNpcConversation GetDaysSinceLastSpokenToNpc(MaryCharacterId character_id);

/*
 * Records a conversation attempt with an NPC and resets days-since-last-spoken
 * to zero. On the first call for an NPC that has not been met, it sets only the
 * met flag. Once the NPC has already been met, it instead sets both the
 * spoken-today and spoken-just-now flags.
 * Parameter: character_id selects the NPC.
 *
 * 记录一次与 NPC 的交谈尝试，并把上次交谈后经过天数重置为零。若该 NPC 尚未
 * 见过，第一次调用只设置已见面标志；已经见过之后再次调用，才会同时设置今日
 * 交谈和刚刚交谈标志。
 * 参数：character_id 选择 NPC。
 */
void MarkNpcSpokenTo(MaryCharacterId character_id);

/*
 * Tests whether the player has spoken to an NPC today.
 * Parameter: character_id selects the NPC.
 * Return value: nonzero when spoken to today; zero otherwise or for an invalid ID.
 *
 * 判断玩家今天是否与某 NPC 交谈过。
 * 参数：character_id 选择 NPC。
 * 返回值：今天交谈过时为非零；否则或 ID 无效时为零。
 */
MaryBool WasNpcSpokenToToday(MaryCharacterId character_id);

/*
 * Tests whether an NPC was spoken to in the engine's current interaction window.
 * Parameter: character_id selects the NPC.
 * Return value: nonzero when marked as just spoken to; zero otherwise.
 *
 * 判断某 NPC 是否在引擎当前交互窗口内刚刚被交谈过。
 * 参数：character_id 选择 NPC。
 * 返回值：被标记为刚刚交谈过时为非零，否则为零。
 */
MaryBool WasNpcSpokenToJustNow(MaryCharacterId character_id);

/*
 * Tests whether the player has met an NPC.
 * Parameter: character_id selects the NPC.
 * Return value: nonzero when met; zero otherwise or for an invalid ID.
 *
 * 判断玩家是否已经见过某 NPC。
 * 参数：character_id 选择 NPC。
 * 返回值：已经见过时为非零；否则或 ID 无效时为零。
 */
MaryBool HasMetNpc(MaryCharacterId character_id);

/*
 * Records that an NPC has received a gift today and resets
 * days-since-last-spoken to zero. It does not set the met or spoken flags.
 * Parameter: character_id selects the NPC. This updates the state queried by
 * WasNpcGiftedToday.
 *
 * 记录某 NPC 今天已经收到礼物，并把上次交谈后经过天数重置为零；它不会设置
 * 已见面或交谈标志。
 * 参数：character_id 选择 NPC。本操作会更新 WasNpcGiftedToday 查询的状态。
 */
void MarkNpcGifted(MaryCharacterId character_id);

/*
 * Tests whether an NPC has already received a gift today.
 * Parameter: character_id selects the NPC.
 * Return value: nonzero when gifted today; zero otherwise or for an invalid ID.
 *
 * 判断某 NPC 今天是否已经收到礼物。
 * 参数：character_id 选择 NPC。
 * 返回值：今天已收礼时为非零；否则或 ID 无效时为零。
 */
MaryBool WasNpcGiftedToday(MaryCharacterId character_id);

/*
 * Gets a romance candidate's love points.
 * Parameter: character_id selects the target-specific candidate.
 * Return value: the current love value, or zero when the ID is not a romance
 * candidate. FOMT resolves bachelorettes; MFOMT resolves bachelors.
 *
 * 获取恋爱候选人的爱情度点数。
 * 参数：character_id 选择目标版本中的恋爱候选人。
 * 返回值：当前爱情度；ID 不是恋爱候选人时返回零。FOMT 对应女性候选人，
 * MFOMT 对应男性候选人。
 */
MaryCharacterLoveValue GetCharacterLove(MaryCharacterId character_id);

/*
 * Adds a signed amount to a romance candidate's love points.
 * Parameters: character_id selects the target-specific candidate; amount is
 * the signed adjustment. This is the modifying counterpart of GetCharacterLove.
 * The native 32-bit sum is limited to 0-65535, not the friendship range
 * 0-255. Extreme adjustments can overflow before these limit checks.
 *
 * 对恋爱候选人的爱情度点数增加一个有符号数值。
 * 参数：character_id 选择目标版本中的恋爱候选人；amount 为有符号调整量。
 * 本函数是 GetCharacterLove 对应的修改操作。
 * 原生 32 位加法结果限制到 0-65535，并非友好度的 0-255；极端调整量
 * 可能在范围检查前溢出。
 */
void AddCharacterLove(MaryCharacterId character_id, MaryCharacterLoveDelta amount);

/*
 * Replaces a romance candidate's love points with an exact value.
 * Parameters: character_id selects the target-specific candidate; love is the
 * new absolute value. Unlike AddCharacterLove(), this does not apply a delta.
 * Only the low 16 bits are stored: 65536 becomes 0 and -1 becomes 65535.
 * Invalid or non-romance character IDs leave the candidate table unchanged.
 *
 * 将恋爱候选人的爱情度直接替换为指定值。
 * 参数：character_id 选择目标版本中的恋爱候选人；love 为新的绝对值。与
 * AddCharacterLove() 不同，本函数不会把参数当作增量。无效或非恋爱候选人的
 * ID 不会修改候选人表。
 * 仅写入低 16 位：65536 变成 0，-1 变成 65535，并非封顶处理。
 */
void SetCharacterLove(MaryCharacterId character_id, MaryCharacterLoveValue love);

/*
 * Assigns an event script to a live scene entity.
 * Parameters: entity_id selects the scene entity; script_id is a symbol from
 * fomt_scripts.mary.h or the exact target-specific script-table ID.
 * entity_id uses MaryEntityId (ENTITY_*), not MaryCharacterId (CHARACTER_*):
 * it addresses a live scene entity rather than the character-name table.
 * For NPC entities, stores the low 16 bits as a script override, without
 * executing the script immediately. Zero selects the entity's default script
 * instead of overriding it. Entities 70-73 instead store a record's low-16-bit
 * script value directly; their getter has no zero-to-default fallback in any
 * of the four targets. Other actor classes may ignore this operation;
 * do not assume every entity supports script binding. Use a valid entity ID:
 * an empty-slot check is not an out-of-range index check.
 *
 * 为当前场景中的实体绑定事件脚本。
 * 参数：entity_id 选择场景实体；script_id 为 fomt_scripts.mary.h 中的符号或
 * 目标版本脚本表的精确 ID。entity_id 使用 MaryEntityId（ENTITY_*），
 * 而不是 MaryCharacterId（CHARACTER_*）；它指向运行时场景实体，并非人物名称表。
 * 对 NPC 实体，将脚本 ID 的低 16 位保存为覆盖值，不会立即执行该脚本。
 * 零表示使用实体默认脚本，而不是覆盖它。实体 70-73 则直接保存记录中的
 * 低 16 位脚本值；四个版本的读取函数均没有零值回退到默认脚本的逻辑。
 * 其他角色实体类型可能忽略此操作，
 * 不能假定所有实体都支持脚本绑定。须使用有效实体 ID；空槽检查不等于越界检查。
 */
void SetEntityEventScript(MaryEntityId entity_id, MaryScriptId script_id);

/*
 * Clears the event script assigned to a live scene entity.
 * Parameter: entity_id selects the same scene-entity domain used by
 * SetEntityEventScript.
 * NPC entities clear the override to zero, restoring selection of their
 * default script. This does not necessarily disable interaction or erase
 * the default script, and is not a command to abort an already running script.
 * Entities 70-73 instead clear their record's script halfword directly;
 * their getter returns zero without selecting a default. Other entity types
 * may ignore the setter. Use a valid entity ID, as for SetEntityEventScript.
 *
 * 清除当前场景实体已绑定的事件脚本。
 * 参数：entity_id 与 SetEntityEventScript 使用同一个运行时场景实体编号域。
 * NPC 实体把覆盖值清零，从而恢复选择默认脚本；这不一定禁用交互，也不会
 * 删除默认脚本，更不是用于中止已运行脚本的指令。
 * 实体 70-73 则直接清零记录中的脚本半字；读取时返回零，不选择默认脚本。
 * 其他实体类型可能忽略该设置。与 SetEntityEventScript 一样，须使用有效实体 ID。
 */
void ClearEntityEventScript(MaryEntityId entity_id);

/*
 * Opens the supermarket's main shopping interface and waits until it closes.
 * Individual staple purchases are completed by PurchaseSupermarketItem.
 * Parameters: none.
 *
 * 打开杂货店主购物界面并等待其关闭。具体常备商品的购买由
 * PurchaseSupermarketItem 完成。
 * 参数：无。
 */
void OpenSupermarketShop(void);

/*
 * Runs the quantity-purchase flow for one of the supermarket's seven staple
 * products.
 * Parameter: item_id is SUPERMARKET_ITEM_* or the exact original
 * integer. The item order is shared by all four verified targets.
 *
 * 执行杂货店七种常备商品之一的数量购买流程。参数 item_id 为
 * SUPERMARKET_ITEM_* 或精确原始整数；四个已验证目标共用同一商品顺序。
 * 参数：item_id 为 SUPERMARKET_ITEM_* 或精确原始整数。
 */
void PurchaseSupermarketItem(MarySupermarketItemId item_id);

/*
 * Opens Won's merchant interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Won 的商店界面并等待其关闭。
 * 参数：无。
 */
void OpenWonShop(void);

/*
 * Opens Gotz's carpenter and farm-upgrade interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Gotz 的木工与农场升级界面并等待其关闭。
 * 参数：无。
 */
void OpenCarpenterShop(void);

/*
 * Opens Saibara's blacksmith interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Saibara 的锻冶屋界面并等待其关闭。
 * 参数：无。
 */
void OpenBlacksmithShop(void);

/*
 * Opens the clinic's examination and medicine interface and waits until it
 * closes.
 * Parameters: none.
 *
 * 打开诊所的诊察与药品界面并等待其关闭。
 * 参数：无。
 */
void OpenClinicShop(void);

/*
 * Opens Kai's seasonal beach cafe interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Kai 的夏季海之家商店界面并等待其关闭。
 * 参数：无。
 */
void OpenBeachCafeShop(void);

/*
 * Opens Barley's Yodel Ranch shopping interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Barley 的 Yodel Ranch 商店界面并等待其关闭。
 * 参数：无。
 */
void OpenYodelRanchShop(void);

/*
 * Opens Manna's winery shopping interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Manna 的果树园商店界面并等待其关闭。
 * 参数：无。
 */
void OpenWineryShop(void);

/*
 * Opens Doug's inn food-ordering interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Doug 的旅馆点餐界面并等待其关闭。
 * 参数：无。
 */
void OpenInnShop(void);

/*
 * Opens Lillia's poultry-farm shopping interface and waits until it closes.
 * Parameters: none.
 *
 * 打开 Lillia 的养鸡场商店界面并等待其关闭。
 * 参数：无。
 */
void OpenPoultryFarmShop(void);

/*
 * Opens the visiting special merchant's shopping interface and waits until it
 * closes. This neutral name is intentional because the localized character
 * label at the same ID differs between targets.
 * Parameters: none.
 *
 * 打开来访特殊商人的购物界面并等待其关闭。由于同一人物 ID 的本地化名称在
 * 不同目标间存在差异，此处有意使用中性的功能名称。
 * 参数：无。
 */
void OpenSpecialMerchantShop(void);

/*
 * Opens the supermarket gift-wrapping item-selection interface and waits until
 * it closes. The surrounding script performs the 100G availability check.
 * Parameters: none.
 *
 * 打开杂货店礼物包装的物品选择界面并等待其关闭。外围脚本负责检查是否有
 * 足够的 100G。
 * 参数：无。
 */
void OpenGiftWrappingMenu(void);

/*
 * Opens a numbered reference page in the game's modal page viewer. page_id is
 * a direct index into the selected ROM's top-level reference-page pointer
 * table. FoMT has 137 ordinary entries (0..136), while MFoMT has 190
 * (0..189). Each entry points to a NULL-terminated list of text pointers; the
 * same physical domain contains tutorials, books, directories, letters,
 * notices, and achievement pages. US and JP use the same index range within
 * each game family, but the strings remain target-specific. The native viewer
 * also recognizes internal pseudo-page values 0x1000..0x1002; vanilla event
 * scripts do not pass those values here. MaryReferencePageId models the shared
 * prefix and the protagonist-version-specific tail of all four tables.
 * Parameter: page_id is a REFERENCE_PAGE_* value for the selected target.
 *
 * 在游戏的模态页面查看器中打开指定编号的资料页。page_id 直接索引所选 ROM 的
 * 顶层资料页指针表：FoMT 有 137 个普通条目（0..136），MFoMT 有 190 个
 * （0..189）。每个条目再指向一个以空指针结束的文本指针列表；教程、书籍、
 * 电话簿、信件、通知及成就页实际共用这张物理表。同一游戏族的 US/JP 使用相同
 * 索引范围，但字符串内容仍属于各自目标。原生查看器还识别 0x1000..0x1002
 * 三个内部伪页面值；原版事件脚本不会在这里传入这些值。MaryReferencePageId
 * 已分别建模四版共用前缀和按男女版变化的后半段。
 * 参数：page_id 为所选目标的 REFERENCE_PAGE_* 值。
 */
void ShowReferencePage(MaryReferencePageId page_id);

/*
 * Opens the farmhouse bookshelf's collected-book list and waits until it
 * closes.
 * Parameters: none.
 *
 * 打开自宅书架的藏书列表并等待其关闭。
 * 参数：无。
 */
void OpenBookList(void);

/*
 * Opens the farmhouse bookshelf's received-letter list and waits until it
 * closes. Scripts check that at least one letter exists before calling it.
 * Parameters: none.
 *
 * 打开自宅书架的收信列表并等待其关闭。脚本会在调用前检查至少存在一封信。
 * 参数：无。
 */
void OpenLetterList(void);

/*
 * Opens the farmhouse calendar interface and waits until it closes. The same
 * callable is used by the calendar entity in every farmhouse upgrade stage.
 * Parameters: none.
 *
 * 打开自宅日历界面并等待其关闭。各个自宅扩建阶段的日历实体共用本函数。
 * 参数：无。
 */
void OpenCalendar(void);

/*
 * Opens the farmhouse shelf article-storage interface and waits for it to
 * close. Parameters: none. The native constructor uses ShelfUi; MFoMT assigns
 * it modal task kind 0x1A.
 *
 * 打开自宅置物架的物品收纳界面并等待其关闭。
 * 参数：无。原生构造器使用 ShelfUi；MFoMT 将其分配为模态任务类型 0x1A。
 */
void OpenShelf(void);

/*
 * Opens the farmhouse tool-chest interface and waits for it to close.
 * Parameters: none. The native constructor uses ToolChestUi; MFoMT assigns it
 * modal task kind 0x1B.
 *
 * 打开自宅工具箱界面并等待其关闭。
 * 参数：无。原生构造器使用 ToolChestUi；MFoMT 将其分配为模态任务类型 0x1B。
 */
void OpenToolChest(void);

/*
 * Opens the farmhouse refrigerator food-storage interface and waits for it to
 * close. Parameters: none. The native constructor uses FridgeUi; MFoMT assigns
 * it modal task kind 0x1C.
 *
 * 打开自宅冰箱的食品收纳界面并等待其关闭。
 * 参数：无。原生构造器使用 FridgeUi；MFoMT 将其分配为模态任务类型 0x1C。
 */
void OpenRefrigerator(void);

/*
 * Opens the farmhouse clock interface and waits until it closes. The same
 * callable is used by the clock entity in every farmhouse upgrade stage.
 * Parameters: none.
 *
 * 打开自宅时钟界面并等待其关闭。各个自宅扩建阶段的时钟实体共用本函数。
 * 参数：无。
 */
void OpenClock(void);

/*
 * Opens the farmhouse kitchen's cooking interface and waits until it closes.
 * Parameters: none.
 *
 * 打开自宅厨房的料理界面并等待其关闭。
 * 参数：无。
 */
void OpenCookingMenu(void);

/*
 * Opens the player's learned-recipe list and waits until it closes.
 * Parameters: none.
 *
 * 打开玩家已经学会的菜谱列表并等待其关闭。
 * 参数：无。
 */
void OpenRecipeList(void);

/*
 * Runs the Game Boy Advance/GameCube communication interface and waits for a
 * terminal MaryGameCubeLinkResult. All four VM wrappers use the same four
 * result classes. SUCCESS is the only result for which the vanilla Harvest
 * Goddess script imports received link milestones. CANCELED_OR_FAILED covers
 * the ordinary unsuccessful and user/partner cancellation paths;
 * INCOMPATIBLE_SAVE_DATA is selected by the native state that displays the
 * two games' data-incompatibility message; UNAVAILABLE is the wrapper fallback
 * when no completed task object exists.
 * Return value: one of the GAMECUBE_LINK_RESULT_* values described above.
 * Parameters: none.
 *
 * 运行 Game Boy Advance/GameCube 通信界面，并等待终态
 * MaryGameCubeLinkResult。四版 VM 包装层使用相同的四类结果。
 * SUCCESS 是唯一会让原版女神脚本导入联机里程碑的结果；
 * CANCELED_OR_FAILED 覆盖普通失败及玩家／联机对方取消路径；
 * INCOMPATIBLE_SAVE_DATA 由显示两个游戏存档数据不兼容提示的原生
 * 状态选中；UNAVAILABLE 是没有已结束任务对象时的包装层后备值。
 * 返回值：上述 GAMECUBE_LINK_RESULT_* 之一。
 * 参数：无。
 */
MaryGameCubeLinkResult RunGameCubeLink(void);

/*
 * Opens the name-entry interface for the selected target kind.
 * Parameter:
 * kind is NAME_ENTRY_* or an exact numeric kind; target_index selects the
 * animal slot for animal births and is NAME_ENTRY_SINGLETON_SLOT for the
 * horse, child, and custom spouse-nickname forms. Native construction preserves
 * this target byte and the completed input task carries it back unchanged.
 * The call completes after the entered name has been stored by the engine.
 *
 * 为指定目标类型打开命名输入界面。参数 kind 为 NAME_ENTRY_* 或精确数字类型；
 * target_index 在动物出生时选择动物槽位，马、孩子及自定义配偶昵称形式使用
 * NAME_ENTRY_SINGLETON_SLOT。原生构造会保留该目标字节，输入完成后的任务也会
 * 将其原样带回。调用会在引擎保存输入名称后完成。
 * 参数：kind 选择 NAME_ENTRY_*；target_index 选择动物槽，单例目标使用
 * NAME_ENTRY_SINGLETON_SLOT。
 */
void OpenNameEntry(MaryNameEntryKind kind, MaryNameEntryTargetIndex target_index);

/*
 * Starts the farm-inheritance flashback shown during Thomas's opening
 * explanation. FoMT-US/JP use this callable in the opening scripts. The
 * corresponding MFoMT slot is unused by the vanilla scripts, but its native
 * constructor is instruction-for-instruction equivalent and selects the same
 * region-specific scene state (US 0x29, JP 0x28), so it is the same retained
 * interface rather than an unidentified modal screen.
 * Parameters: none.
 *
 * 启动 Thomas 在开场说明牧场继承经过时使用的回忆场景。FoMT-US/JP
 * 的开场脚本会调用它。MFoMT 原版脚本虽未引用对应槽，但其原生构造器
 * 与 FoMT 指令级等价，且选择相同的地区场景状态（US 0x29、JP 0x28），
 * 因此这是保留的同一接口，而非无法识别的模态界面。
 * 参数：无。
 */
void StartFarmInheritanceFlashback(void);

/*
 * Opens the reusable on-screen keyboard used by the naming sequence and waits
 * for editing to finish. The MFoMT-US/JP task implementations create a
 * 31-byte edit buffer, character-selection state machine, and commit the
 * resulting string back to the naming workflow. FoMT exposes the same stage at
 * raw callable slot 0x0A5; MFoMT uses 0x0A8.
 * Parameters: none.
 *
 * 打开命名流程复用的屏幕键盘，并等待编辑完成。MFoMT-US/JP 的任务实现会创建
 * 31 字节编辑缓冲区和字符选择状态机，最后把结果字符串写回命名流程。FoMT 的
 * 原始 callable 槽为 0x0A5，MFoMT 为 0x0A8。
 * 参数：无。
 */
void OpenNameEntryKeyboard(void);

/*
 * Opens the player's full rucksack interface and waits until it closes. ROM
 * code constructs separate "Tools" and "Items" panels, their cursors, and the
 * item-transfer controls. The surrounding system script then checks whether a
 * rucksack action such as using the Teleport Stone changed the current map.
 *
 * unused_stack_value preserves an original bytecode operand (vanilla passes
 * 1). All four native handlers leave it unconsumed and the rucksack task never
 * reads it, so it must not be interpreted as a menu mode or Boolean option.
 * Parameter: unused_stack_value preserves that original stack operand; the
 * native handler does not consume its value.
 *
 * 打开玩家的完整背包界面并等待关闭。ROM 代码会分别构造“Tools”和“Items”
 * 面板、光标和物品交换控制；外围系统脚本随后检查使用飞行石等背包操作是否改变
 * 了当前地图。
 *
 * unused_stack_value 只用于保留原始字节码操作数（原版传入 1）。四个 native
 * handler 均不消费该值，背包任务也不会读取它，因此不能将其解释成菜单模式或
 * 布尔选项。
 * 参数：unused_stack_value 保留原始栈操作数；原生 handler 不消费其值。
 */
void OpenRucksackMenu(int unused_stack_value);

/*
 * Opens the festival-entry selector for one livestock family.
 * Parameter: festival_kind is FESTIVAL_ANIMAL_*.
 * Return value: the selected zero-based animal slot, or -1 when selection is
 * cancelled.
 *
 * 打开指定家畜类别的祭典参赛选择界面。
 * 参数：festival_kind 为 FESTIVAL_ANIMAL_*。
 * 返回值：所选动物从 0 开始的槽位；取消选择时为 -1。
 */
MaryAnimalSlotIndex SelectFestivalAnimal(MaryFestivalAnimalKind festival_kind);

/*
 * Runs the staff-credits sequence and waits for it to finish. The parameterless
 * handler installs native state selector 0x2E in US or 0x2D in JP, replaces the
 * active state object, writes transition value 24, and yields through the VM
 * lifecycle hook. The installed task transfers the wrapped state object into a
 * dedicated 0x730-byte credits viewer whose constructor binds
 * gStaffCreditsLines; its update state machine draws successive credit lines
 * and performs the palette transitions. Every vanilla call follows
 * FadeOutScreen. Callers include weddings, the MFoMT opening refusal ending,
 * and Thomas's final GameCube cooking-record joke, so the operation is a
 * reusable credits sequence rather than a wedding-only transition. The
 * regional selector difference belongs to the native state enum and is not
 * exposed by Mary.
 * Parameters: none.
 *
 * 运行职员表序列并等待其结束。该无参数处理器安装原生状态选择值（US 为 0x2E、
 * JP 为 0x2D），替换当前状态对象，写入转换值 24，并通过 VM 生命周期 hook 让出
 * 执行。安装的任务会把包装状态对象转交给专用的 0x730 字节职员表查看器；其构造器
 * 直接绑定 gStaffCreditsLines，更新状态机逐行绘制职员名单并执行调色板过渡。原版
 * 全部调用都位于 FadeOutScreen 之后，调用场景包括婚礼、MFoMT 开场拒绝继承牧场的
 * 结局，以及 Thomas 最后一段 GameCube 料理记录彩蛋，因此它是可复用的职员表序列，
 * 而非婚礼专用转换。地区 selector 差异属于原生状态枚举内部，Mary 不对外暴露。
 * 参数：无。
 */
void RunStaffCredits(void);

/*
 * Opens one of Thomas's interactive farming tutorials and waits until it
 * closes.
 * Parameter: tutorial_kind is FARMING_TUTORIAL_*.
 *
 * 打开 Thomas 提供的一项交互式农场教程并等待其关闭。参数 tutorial_kind 为
 * FARMING_TUTORIAL_*。
 * 参数：tutorial_kind 为 FARMING_TUTORIAL_*。
 */
void OpenFarmingTutorial(MaryFarmingTutorialKind tutorial_kind);

/*
 * Runs the clock-menu preparation hook before fade-out and again after the
 * modal clock closes. Parameters: none. It is distinct from the cooking and
 * recipe hooks in all four native virtual-method tables. All four ROMs invoke
 * virtual callback 0x84 on the active scene controller.
 *
 * 在渐隐前以及时钟模态界面关闭后运行时钟界面准备钩子。
 * 参数：无。四个 ROM 均调用当前场景控制器虚函数表偏移 0x84 的回调；原生
 * 虚函数表将其与料理、菜谱钩子分开保存。
 */
void PrepareClockMenuTransition(void);

/*
 * Runs the clock-menu restoration hook after the field view is restored.
 * Parameters: none. In all four ROMs this invokes virtual callback 0x88 on
 * the active scene controller, paired with the separate clock preparation
 * path at callback 0x84. It does not itself decode input, draw the clock, or
 * reopen the menu.
 *
 * 场景画面恢复后运行时钟界面恢复钩子。
 * 参数：无。四个 ROM 均会在当前场景控制器上调用虚函数表偏移 0x88 的回调，
 * 与偏移 0x84 的时钟准备流程配对。本函数本身不解析输入、不绘制时钟，也不
 * 重新打开菜单。
 */
void RestoreAfterClockMenu(void);

/*
 * Runs the cooking-menu preparation hook before fade-out and again after the
 * modal cooking interface closes. Parameters: none. All four ROMs dispatch
 * virtual callback 0x8C on the active scene controller.
 *
 * 在渐隐前以及料理模态界面关闭后运行料理界面准备钩子。
 * 参数：无。四个 ROM 均调用当前场景控制器虚函数表偏移 0x8C 的回调。
 */
void PrepareCookingMenuTransition(void);

/*
 * Runs the cooking-menu restoration hook after the field view is restored.
 * Parameters: none. All four ROMs dispatch virtual callback 0x90, paired
 * with PrepareCookingMenuTransition at callback 0x8C.
 *
 * 场景画面恢复后运行料理界面恢复钩子。
 * 参数：无。四个 ROM 均调用虚函数表偏移 0x90 的回调，与偏移 0x8C 的
 * PrepareCookingMenuTransition 配对。
 */
void RestoreAfterCookingMenu(void);

/*
 * Runs the recipe-list preparation hook before fade-out and again after the
 * modal recipe list closes. Parameters: none. All four ROMs dispatch virtual
 * callback 0x94 on the active scene controller.
 *
 * 在渐隐前以及菜谱列表模态界面关闭后运行菜谱界面准备钩子。
 * 参数：无。四个 ROM 均调用当前场景控制器虚函数表偏移 0x94 的回调。
 */
void PrepareRecipeMenuTransition(void);

/*
 * Runs the recipe-list restoration hook after the field view is restored.
 * Parameters: none. All four ROMs dispatch virtual callback 0x98, paired
 * with PrepareRecipeMenuTransition at callback 0x94.
 *
 * 场景画面恢复后运行菜谱列表恢复钩子。
 * 参数：无。四个 ROM 均调用虚函数表偏移 0x98 的回调，与偏移 0x94 的
 * PrepareRecipeMenuTransition 配对。
 */
void RestoreAfterRecipeMenu(void);

/*
 * Tests whether an album is currently inserted in the farmhouse record player.
 * Parameters: none.
 * Return value: nonzero when the record player's one-bit has_album field is
 * set; zero otherwise. This is unrelated to Van's album-unlock inventory.
 * Record-player interaction scripts use it before selecting or playing music.
 *
 * 判断农舍唱片机当前是否实际放入了一张唱片。
 * 参数：无。
 * 返回值：唱片机的一位 has_album 字段置位时为非零，否则为零。该状态与 Van
 * 商店的唱片解锁库存无关；唱片机交互脚本会在选择或播放音乐前调用本函数。
 */
MaryBool RecordPlayerHasAlbum(void);

/*
 * Inserts an album article into the farmhouse record player and returns the
 * article ID of the album that was ejected. If the supplied article is not an
 * album, no album is inserted and the returned article slot is empty.
 * Parameter: article_id is the held article ID, normally one of Album 1-15.
 * Return value: for a valid album, the article ID represented by the previous
 * stored album index. When an album was already present, this is the ejected
 * album. When the player inserts the first album into an empty record player,
 * the native object still returns ITEM_ARTICLE_ALBUM_1 from its initialized index;
 * the vanilla script intentionally ignores that return on the empty branch.
 * A non-album input leaves the player unchanged and returns ITEM_ARTICLE_NONE.
 * The occupied branch removes the held article first, then gives the returned
 * prior album back to the player.
 *
 * 将一张唱片类物品放入农舍唱片机，并返回被替换出来的旧唱片物品 ID。
 * 如果传入的物品不是唱片，则不会放入唱片，返回的物品槽为空。
 * 参数：article_id 为手持物品 ID，正常取值为唱片 1 至唱片 15。
 * 返回值：合法唱片会返回原先保存的唱片索引所代表的物品 ID。原本已有唱片时，
 * 这就是被弹出的唱片；唱片机为空且首次放入唱片时，原生对象仍会从初始化为零的
 * 旧索引返回 ITEM_ARTICLE_ALBUM_1，原版脚本在空分支有意忽略该返回值。传入非唱片
 * 不会改变唱片机，并返回 ITEM_ARTICLE_NONE。已有唱片的分支会先移除手持物，再把
 * 返回的旧唱片交还给玩家。
 */
MaryItemArticleId SwapRecordPlayerAlbum(MaryItemArticleId article_id);

/*
 * Removes the album currently inserted in the farmhouse record player.
 * Parameters: none.
 * Return value: the removed album article ID, or the engine's empty-slot value
 * when no album is inserted. The interaction script gives this value to the
 * player as the newly held article.
 *
 * 取出农舍唱片机中当前放置的唱片。
 * 参数：无。
 * 返回值：取出的唱片物品 ID；没有唱片时返回引擎使用的空物品槽值。交互脚本
 * 会把该返回值作为玩家新拿起的物品。
 */
MaryItemArticleId RemoveRecordPlayerAlbum(void);

/*
 * Attempts to light the fireplace associated with a map. The matching four-ROM
 * query operation proves that MAP_MOUNTAIN_COTTAGE, MAP_FARMHOUSE, and
 * MAP_TOWN_COTTAGE are the three readable fireplace states; this setter delegates
 * map_id to the native location-state dispatcher. The FoMT source proves that the
 * farmhouse accepts the change only when its upgrade level is greater than 1;
 * otherwise that branch has no effect. The farmhouse lit flag is cleared by its
 * daily update.
 * Parameter: map_id is one of the three supported MaryMapId values above.
 *
 * 尝试点燃指定地图关联的壁炉。与其配套的四版查询操作证明，可读取的壁炉状态
 * 只有 MAP_MOUNTAIN_COTTAGE、MAP_FARMHOUSE 与 MAP_TOWN_COTTAGE 三项；本
 * setter 会把 map_id 交给原生地点状态分派器。FoMT 源码证明，农舍分支仅在
 * 扩建等级大于 1 时接受点燃操作，否则无效果；农舍每日更新会清除其点燃标志。
 * 参数：map_id 为上述三个受支持的 MaryMapId 之一。
 */
void LightFireplaceAtLocation(MaryMapId map_id);

/*
 * Tests whether the fireplace associated with a map is lit. All four native
 * wrappers dispatch exactly MAP_MOUNTAIN_COTTAGE, MAP_FARMHOUSE, and
 * MAP_TOWN_COTTAGE; every other map ID takes the explicit false branch.
 * Parameter: map_id is one of the three supported MaryMapId values above.
 * Return value: TRUE when lit; FALSE when unlit or when map_id is unsupported.
 *
 * 判断指定地图关联的壁炉是否已经点燃。四个原生包装函数均只分派
 * MAP_MOUNTAIN_COTTAGE、MAP_FARMHOUSE 与 MAP_TOWN_COTTAGE；其他地图 ID
 * 明确进入返回 FALSE 的分支。
 * 参数：map_id 为上述三个受支持的 MaryMapId 之一。
 * 返回值：已点燃时为 TRUE；未点燃或 map_id 不受支持时为 FALSE。
 */
MaryBool IsFireplaceLitAtLocation(MaryMapId map_id);

/*
 * Places an article in the farmhouse vase and initializes its vase lifespan.
 * Parameter: article_id is normally one of ITEM_ARTICLE_FLOWER_*; passing the exact
 * integer remains supported for byte-identical compilation. Vase interaction
 * scripts obtain this value from the held article and pair this operation with
 * GetVaseArticleId.
 *
 * 将物品放入农舍花瓶，并初始化该物品在花瓶中的保存期限。
 * 参数：article_id 通常为 ITEM_ARTICLE_FLOWER_*；也可继续传入精确整数，编译字节
 * 保持一致。花瓶交互脚本从手持物品取得该值，并与 GetVaseArticleId 配合使用。
 */
void SetVaseArticleId(MaryItemArticleId article_id);

/*
 * Gets the article currently placed in the farmhouse vase.
 * Parameters: none.
 * Return value: ITEM_ARTICLE_* or the exact article ID; ITEM_ARTICLE_NOT_PRESENT when
 * the vase is empty. The FoMT-US native wrapper compares FarmHouse's physical
 * ITEM_ARTICLE_NONE value and maps it to -1 before returning to the VM; the MFoMT
 * wrapper and both regional script sets use the same script-facing sentinel.
 *
 * 获取农舍花瓶中当前放置的物品。
 * 参数：无。
 * 返回值：ITEM_ARTICLE_* 或精确物品 ID；花瓶为空时返回 ITEM_ARTICLE_NOT_PRESENT。
 * FoMT-US 原生包装器会比较 FarmHouse 的物理 ITEM_ARTICLE_NONE，并在返回 VM 前
 * 映射成 -1；MFoMT 包装器及两个地区的脚本使用同一个脚本侧哨兵。
 */
MaryItemArticleId GetVaseArticleId(void);

/*
 * Tests whether a chicken-coop feed trough is already filled.
 * Parameter: trough_index uses CHICKEN_COOP_FEED_TROUGH_SLOT_*; only slots
 * 01-04 are available before the coop upgrade and all eight afterwards.
 * Return value: 1 when feed is present; 0 when empty or beyond current
 * capacity. This query does not trigger the setter's map-tile refresh.
 * Feed-box inspection scripts pair this with FillChickenFeedTrough.
 *
 * 判断鸡舍中的指定饲料槽是否已经放入饲料。
 * 参数：trough_index 使用 CHICKEN_COOP_FEED_TROUGH_SLOT_*；扩建前仅槽位
 * 01-04 可用，扩建后八个全部可用。
 * 返回值：已有饲料时为 1；为空或超出当前容量时为 0。本查询不触发设置函数
 * 的地图图块刷新。饲料箱检查脚本会将其与
 * FillChickenFeedTrough 配对使用。
 */
MaryBool IsChickenFeedTroughFilled(MaryChickenCoopFeedTroughIndex trough_index);

/*
 * Fills a chicken-coop feed trough.
 * Parameter: trough_index uses CHICKEN_COOP_FEED_TROUGH_SLOT_* and is checked
 * against the current four- or eight-trough capacity.
 * Call IsChickenFeedTroughFilled first when the script must avoid replacing
 * feed that is already present.
 * In all four targets the storage setter only sets a feed bit, without consuming
 * inventory. In all four targets the later map refresh indexes an X-coordinate
 * table with the original trough index, without its own capacity check.
 * Always pass a valid trough index; do not rely on invalid input being a no-op.
 *
 * 向鸡舍中的指定饲料槽放入饲料。
 * 参数：trough_index 使用 CHICKEN_COOP_FEED_TROUGH_SLOT_*，并按当前四个或
 * 八个槽的容量校验。若脚本需要避免覆盖已有饲料，应先调用
 * IsChickenFeedTroughFilled。
 * 四个版本的存储设置函数均只设置饲料位，不扣库存。后续地图刷新
 * 均用原始槽位编号索引 X 坐标表，没有独立容量检查。必须传入有效编号，不可依赖
 * 越界输入会安全地不执行任何操作。
 */
void FillChickenFeedTrough(MaryChickenCoopFeedTroughIndex trough_index);

/*
 * Begins egg incubation in the selected coop incubator.
 * Parameter: incubator_index uses CHICKEN_COOP_INCUBATOR_SOUTH or
 * CHICKEN_COOP_INCUBATOR_NORTH. Only the south incubator exists before the upgrade.
 * Check IsIncubatorOccupied before calling when replacement is not intended.
 * The four native storage setters mark the incubator occupied and set its
 * countdown to 3, even if it was already occupied. Egg validation and removal
 * belong to the surrounding script, not this storage setter.
 * An out-of-capacity index skips the storage write, but the outer context
 * method still forwards that index to its refresh callback. Use a valid
 * incubator index; the storage guard is not a whole-call no-op guarantee.
 *
 * 在选定的鸡舍孵化箱中开始孵化鸡蛋。
 * 参数：incubator_index 使用 CHICKEN_COOP_INCUBATOR_SOUTH 或
 * CHICKEN_COOP_INCUBATOR_NORTH；扩建前只有南侧孵化箱存在。不希望覆盖时，应先调用
 * IsIncubatorOccupied 检查。
 * 四个版本的原生存储函数都会设置占用标记，并将倒计时设为 3；已经占用时
 * 也会重置。鸡蛋检查与扣除由外围脚本负责，不由该存储函数完成。
 * 超出当前容量的编号会跳过存储写入，但外层上下文方法仍将该编号传给刷新
 * 回调。应使用有效孵化箱编号，不能将存储层保护等同于整个调用毫无影响。
 */
void BeginEggIncubation(MaryChickenCoopIncubatorIndex incubator_index);

/*
 * Tests whether a chicken-coop incubator is occupied.
 * Parameter: incubator_index selects CHICKEN_COOP_INCUBATOR_SOUTH or
 * CHICKEN_COOP_INCUBATOR_NORTH.
 * Return value: TRUE when occupied; FALSE when empty or outside current
 * incubator capacity. FALSE alone does not prove that an index is usable.
 * Incubator inspection scripts use indices 0 and 1.
 *
 * 判断鸡舍中的指定孵化器是否已被占用。
 * 参数：incubator_index 选择 CHICKEN_COOP_INCUBATOR_SOUTH 或
 * CHICKEN_COOP_INCUBATOR_NORTH。
 * 返回值：已占用时为 TRUE；为空或超出当前孵化箱容量时均返回 FALSE，
 * 所以仅凭 FALSE 不能认定编号可用。孵化器检查脚本使用索引 0 和 1。
 */
MaryBool IsIncubatorOccupied(MaryChickenCoopIncubatorIndex incubator_index);

/*
 * Gets the number of usable incubators in the current chicken coop.
 * Parameters: none.
 * Return value: 1 before the coop upgrade, 2 after the upgrade.
 * This reads the coop's one-bit upgrade level; it does not count occupied or
 * empty incubators. All four native implementations use the same rule.
 *
 * 获取当前鸡舍中可用的孵化箱数量。
 * 参数：无。
 * 返回值：鸡舍扩建前为 1，扩建后为 2。
 * 该函数读取鸡舍的一位扩建等级，不统计已占用或空闲孵化箱。四个原生版本使用
 * 相同规则。
 */
MaryChickenCoopIncubatorCapacity GetIncubatorCapacity(void);

/*
 * Tests whether the egg in an incubator has reached its hatch day.
 * Parameter: incubator_index selects CHICKEN_COOP_INCUBATOR_SOUTH or
 * CHICKEN_COOP_INCUBATOR_NORTH and is checked against the current capacity.
 * Return value: nonzero when ready to hatch; zero otherwise.
 *
 * 判断指定孵化箱中的鸡蛋是否已经到达孵化日。
 * 参数：incubator_index 选择 CHICKEN_COOP_INCUBATOR_SOUTH 或
 * CHICKEN_COOP_INCUBATOR_NORTH，并按当前一个或两个孵化箱的容量校验。
 * 返回值：可以孵化时为非零，否则为零。
 */
MaryBool IsEggReadyToHatch(MaryChickenCoopIncubatorIndex incubator_index);

/*
 * Hatches a ready egg and inserts the new chick into the chicken-coop roster.
 * Parameter: incubator_index selects CHICKEN_COOP_INCUBATOR_SOUTH or
 * CHICKEN_COOP_INCUBATOR_NORTH and is checked against the current capacity.
 * Return value: the new chicken's roster slot, or -1 if hatching fails.
 * All four targets clear incubator occupancy before inserting the chick.
 * A later insertion failure does not restore the egg. The readiness check
 * requires an occupied incubator with its countdown at zero.
 * After constructing the chick, all targets add rand() % 10 affection
 * (0-9 points) before insertion. Construction initializes affection to zero,
 * so this is also the newborn's initial total. This is not an event-state ID.
 *
 * 孵化已经到期的鸡蛋，并将新生小鸡加入鸡舍动物列表。
 * 参数：incubator_index 选择 CHICKEN_COOP_INCUBATOR_SOUTH 或
 * CHICKEN_COOP_INCUBATOR_NORTH，并按当前一个或两个孵化箱的容量校验。
 * 返回值：新生小鸡的列表槽位；孵化失败时为 -1。
 * 四个版本都先清除孵化箱占用状态，再将小鸡插入列表；之后插入失败不会
 * 恢复鸡蛋。孵化条件要求孵化箱处于占用状态，且倒计时为零。
 * 四个版本都在构造小鸡后、插入列表前增加 rand() % 10 的好感度（0-9 点）。
 * 构造时好感度初始化为零，因此这也是小鸡的初始总好感度，不是事件状态编号。
 */
MaryAnimalChickenSlotIndex AttemptEggHatch(MaryChickenCoopIncubatorIndex incubator_index);

/*
 * Tests whether a barn feed trough is already filled.
 * Parameter: trough_index uses BARN_FEED_TROUGH_{NORTH,SOUTH}_ROW_SLOT_* for
 * ordinary troughs or BARN_PREGNANCY_FEED_TROUGH_{NORTH,SOUTH} for pregnancy
 * troughs. The handler checks the selected group against current capacity.
 * Return value: 1 when fodder is present; 0 when empty or outside the selected
 * group's current capacity. Unlike FillBarnFeedTrough, this query does not
 * invoke the map coordinate-table refresh path.
 * A physical trough index is not an animal roster index. FoMT-US Barn::DayUpdate
 * pools ordinary trough feed and distributes it to eligible animals; pregnancy
 * feed is first checked against the animal linked to that pregnancy stall.
 *
 * 判断牛羊小屋中的指定饲料槽是否已经放入饲料。
 * 参数：trough_index 使用 BARN_FEED_TROUGH_{NORTH,SOUTH}_ROW_SLOT_* 选择
 * 普通槽，或使用 BARN_PREGNANCY_FEED_TROUGH_{NORTH,SOUTH} 选择怀孕槽；
 * 处理函数会按当前牛羊舍扩建状态校验对应分组的容量。
 * 返回值：已有牧草时为 1；为空或超出对应分组当前容量时为 0。与
 * FillBarnFeedTrough 不同，本查询不会进入地图坐标表刷新路径。
 * 物理饲料槽索引不是动物记录索引。FoMT-US Barn::DayUpdate 汇总普通槽中的
 * 饲料，再分配给符合条件的动物；妊娠槽饲料则先检查该妊娠栏关联的动物。
 */
MaryBool IsBarnFeedTroughFilled(MaryBarnFeedTroughIndex trough_index);

/*
 * Fills a barn feed trough with fodder.
 * Parameter: trough_index uses the same normal-stall and pregnancy-stall
 * encoding as IsBarnFeedTroughFilled. Feed-box scripts normally check the
 * trough first; scripted tutorials may fill it directly.
 * In all four targets, the Barn storage setters only set the selected feed bit; they
 * do not subtract stored fodder. The ordinary feed-box script separately
 * calls UsePlayerHeldItem before FillBarnFeedTrough. Preserve that separation
 * when implementing a player-fed interaction; filling a bit twice does not
 * create two portions of feed.
 * Use a valid trough index: all four targets subsequently forward the original
 * index to the current map's feed-tile refresh. That path indexes coordinate
 * tables without the storage setter's capacity guard; rejected storage
 * indices are not guaranteed to make the entire callable a safe no-op.
 *
 * 向牛羊小屋中的指定饲料槽放入牧草。
 * 参数：trough_index 使用与 IsBarnFeedTroughFilled 相同的普通畜栏和怀孕畜栏
 * 编码。饲料箱脚本通常会先检查饲料槽；教学事件也可能直接放入牧草。
 * 四个版本的 Barn 存储设置函数只设置所选饲料位，不扣减库存牧草。普通饲料箱
 * 脚本会在 FillBarnFeedTrough 之前另行调用 UsePlayerHeldItem。编写玩家投喂
 * 交互时应保留这种分工；同一个饲料位设置两次不会变成两份饲料。
 * 必须使用有效饲料槽编号：四个版本后续仍把原始编号传给当前地图的饲料图块
 * 刷新路径，该路径索引坐标表时没有沿用存储函数的容量检查。因此，存储函数
 * 拒绝越界编号并不保证整个 callable 会安全地不执行任何操作。
 */
void FillBarnFeedTrough(MaryBarnFeedTroughIndex trough_index);

/*
 * Starts the player's shipment-box deposit animation. The held object must
 * already have been shipped or removed by the caller; this callable handles
 * the player-facing animation state rather than inventory or shipment data.
 * It returns immediately, so scripted tutorials add their own delay before
 * continuing.
 * Parameters: none.
 *
 * 启动玩家向出货箱投入物品的动画。调用前，外围脚本必须已经完成物品出货或
 * 移除；本函数只处理玩家角色的动画状态，不修改物品栏或出货数据。
 * 本函数立即返回，因此教学事件会自行等待一段时间后再继续。
 * 参数：无。
 */
void StartShipmentBoxDepositAnimation(void);

/*
 * Gets the number of usable pregnancy stalls in the current barn.
 * Parameters: none.
 * Return value: 1 before the barn upgrade, 2 after the upgrade.
 * This reads the barn's one-bit upgrade level; it does not count occupied or
 * empty pregnancy stalls. All four native implementations use the same rule.
 *
 * 获取当前畜舍中可用的妊娠栏数量。
 * 参数：无。
 * 返回值：畜舍扩建前为 1，扩建后为 2。
 * 该函数读取畜舍的一位扩建等级，不统计已占用或空闲妊娠栏。四个原生版本使用
 * 相同规则。
 */
MaryBarnPregnancyStallCapacity GetPregnancyStallCapacity(void);

/*
 * Tests whether the cow or sheep in a pregnancy stall is ready to give birth.
 * After validating the linked animal, all four targets require healthy
 * pregnancy days > 20 OR total pregnancy days >= 30. The two native getters
 * return zero when the animal is not pregnant; this is not an unconditional
 * 21-calendar-day timer. The shared daily updater increments total pregnancy
 * days every day while pregnant, regardless of feeding or sickness, and
 * increments healthy pregnancy days only when the animal is not sick after
 * that day's livestock health update. Both five-bit counters saturate at 31.
 * Invalid or unoccupied links return FALSE.
 * Parameter: pregnancy_stall_index selects BARN_PREGNANCY_STALL_NORTH or
 * BARN_PREGNANCY_STALL_SOUTH; only the north stall exists before the upgrade.
 * Return value: nonzero when birth is due; zero otherwise.
 *
 * 判断指定妊娠栏中的牛或羊是否已经到达生产日。
 * 四个版本均先验证关联动物，再判断健康妊娠天数大于 20，或总妊娠天数
 * 达到 30。未怀孕时两个原生计数读取函数均返回零，因此不是无条件经过
 * 21 个日历日就可以生产。共用的每日更新器会在怀孕期间每天增加总妊娠天数，
 * 不受是否喂食或生病影响；健康妊娠天数则只在当天家畜健康更新后仍未生病时
 * 增加。两个五位计数均在 31 封顶。关联编号无效或槽位为空时返回 FALSE。
 * 参数：pregnancy_stall_index 选择 BARN_PREGNANCY_STALL_NORTH 或
 * BARN_PREGNANCY_STALL_SOUTH；扩建前只有北侧妊娠栏存在。
 * 返回值：可以生产时为非零，否则为零。
 */
MaryBool IsBarnAnimalReadyToGiveBirth(MaryBarnPregnancyStallIndex pregnancy_stall_index);

/*
 * Delivers a ready cow or sheep and inserts the newborn into the barn roster.
 * Parameter: pregnancy_stall_index selects BARN_PREGNANCY_STALL_NORTH or
 * BARN_PREGNANCY_STALL_SOUTH and is checked against current stall capacity.
 * Return value: the newborn's barn roster slot, or -1 if birth fails.
 * Insertion searches from roster slot zero up to the current barn capacity;
 * the returned index is not the pregnancy-stall index. A full roster returns
 * -1 rather than overwriting an occupied slot.
 * All four native handlers check readiness first, then clear the pregnancy
 * stall's linked animal index before constructing the newborn. Do not assume
 * that a failure return guarantees the original stall association is intact.
 * Both cow and sheep paths reset the parent's pregnancy before attempting
 * roster insertion; the insertion loops can return -1 without rolling back
 * that reset or the cleared link.
 * FoMT-JP differs: its birth paths sample the PRNG without the other targets'
 * low-affection guard. Its sheep path also passes a null pointer to the
 * affection getter, which reads offset 0x18. The extracted byte is halved,
 * used as the divisor of PRNG remainder calculation, and the remainder is
 * written as the newborn sheep's initial affection. Because address 0x18 is
 * protected GBA BIOS space when called from ROM, the byte reflects the most
 * recently fetched BIOS opcode rather than maternal affection; its exact
 * value is execution-history and hardware/emulator dependent. This documents
 * the shipped data flow and must not be treated as a portable initialization
 * rule or silently repaired by the compiler.
 *
 * 让已经到期的牛或羊生产，并将幼崽加入畜舍动物列表。
 * 参数：pregnancy_stall_index 选择 BARN_PREGNANCY_STALL_NORTH 或
 * BARN_PREGNANCY_STALL_SOUTH，并按当前一个或两个妊娠栏的容量校验。
 * 返回值：新生动物的畜舍列表槽位；生产失败时为 -1。
 * 插入时从动物列表的零号槽开始，在当前畜舍容量内查找空位；返回编号不是
 * 妊娠栏编号。列表已满时返回 -1，不会覆盖已占用的动物槽。
 * 四个版本的原生实现均先检查生产条件，再清除妊娠栏的动物关联编号，然后
 * 构造幼崽。因此不能假定返回失败就意味着原栏位关联一定保持不变。
 * 牛和羊的路径都会先重置母体怀孕状态，再尝试插入动物列表；插入循环可以
 * 返回 -1，且不会回滚此前的怀孕重置或栏位关联清除。
 * FoMT-JP 存在差异：生产路径没有其他版本的低好感度随机采样保护；羊的
 * 路径还将空指针传给读取偏移 0x18 的好感度函数。取出的字节先除以 2，
 * 再作为 PRNG 余数运算的除数，所得余数会写成新生羊的初始好感度。由于从
 * ROM 执行时地址 0x18 位于受保护的 GBA BIOS 区域，该字节反映最近一次成功
 * 取出的 BIOS 指令，而不是母体好感度；精确值依赖执行历史以及硬件／模拟器
 * 实现。这里记录的是原版实际数据流，不能把它当成可移植的初始化规则，也不
 * 能由编译器擅自修复。
 */
MaryAnimalSlotIndex AttemptBarnAnimalBirth(MaryBarnPregnancyStallIndex pregnancy_stall_index);

/*
 * Permanently unlocks the mountain cottage awarded by the fiftieth wedding
 * anniversary event.
 * Parameters: none. The native operation idempotently sets bit 0 of the shared
 * persistent facility byte and preserves every other bit; it does not directly
 * construct map entities. The surrounding event sets its own completion
 * variable before invoking this world-state change.
 *
 * 永久解锁结婚五十周年事件奖励的山顶别墅。
 * 参数：无。原生操作会幂等地置位共用持久设施字节的第 0 位并保持其他位不变，
 * 不会直接构造地图实体。外围事件会先设置自身的完成变量，再调用本函数修改世界状态。
 */
void BuildMountainCottage(void);

/*
 * Permanently unlocks the Seaside Cottage on Mineral Beach. The shipped
 * scripts call this after the Harvest Goddess announces the maximum link
 * level reward. The native operation idempotently sets bit 2 of the same
 * persistent facility byte used by BuildMountainCottage and preserves every
 * other bit; it does not directly construct map entities.
 * Parameters: none.
 *
 * 永久解锁矿石海滩的海边别墅。原版脚本在女神宣布联动度满级奖励后调用本函数。
 * 原生操作会幂等地置位 BuildMountainCottage 所用同一持久设施字节的第 2 位并
 * 保持其他位不变，不会直接构造地图实体。
 * 参数：无。
 */
void BuildSeasideCottage(void);

/*
 * Tests whether at least one piece of Golden Lumber is placed on the farm.
 * Parameters: none.
 * Return value: TRUE as soon as one of the farm's complete 25-by-43 FieldPlot
 * grid has a nonzero placed-object state and object ID 0x1A (Golden Lumber),
 * otherwise FALSE after scanning all 1,075 plots. It does not inspect the
 * rucksack, cabinet, shipment history, or a cached event variable. Daily
 * dialogue and achievement scripts use it for the special villager reaction
 * to displaying Golden Lumber.
 *
 * 判断农场中是否至少摆放了一块黄金资材。
 * 参数：无。
 * 返回值：依次扫描农场完整的 25×43 个 FieldPlot；一旦发现摆放对象状态非零且
 * 对象 ID 为 0x1A（黄金资材）便返回 TRUE，扫描完 1075 个地块仍未发现则返回
 * FALSE。它不会检查背包、置物柜、出货历史或缓存的事件变量。每日对话和成就
 * 脚本用它触发村民对展示黄金资材的特殊反应。
 */
MaryBool HasGoldenLumberOnFarm(void);

/*
 * Opens a map door or entrance by its runtime door ID. Scripts pair this with
 * the opening sound and movement through the entrance.
 * Parameter: door_index is the current map's local door/entrance index. It is
 * not a global location identifier. The callable immediately dispatches the
 * current map controller's virtual open operation and does not wait for its
 * animation. If no map controller exists, it is a no-op. Neither the VM
 * wrapper nor the dispatch shim validates the index.
 *
 * 按运行时门编号打开地图中的门或入口。脚本通常将其与开门音效及穿门移动配合。
 * 参数：door_index 为当前地图局部的门／入口索引，并非全局地点编号。本调用会
 * 立即分派当前地图控制器的虚拟开门操作，不等待动画；地图控制器不存在时无操作
 * 返回。VM wrapper 和分派薄层都不会校验索引。
 */
void OpenDoor(MaryDoorIndex door_index);

/*
 * Closes a map door or entrance previously opened by OpenDoor.
 * Parameter: door_index uses the same current-map local domain as OpenDoor.
 * This immediately dispatches the distinct virtual close operation, does not
 * wait for animation, becomes a no-op without a map controller, and performs
 * no index validation in the wrapper or dispatch shim.
 *
 * 关闭此前由 OpenDoor 打开的地图门或入口。
 * 参数：door_index 与 OpenDoor 使用同一当前地图局部索引域。本调用立即分派另一
 * 个虚拟关门操作，不等待动画；地图控制器不存在时无操作返回，wrapper 与分派
 * 薄层均不校验索引。
 */
void CloseDoor(MaryDoorIndex door_index);

/*
 * Redraws the supermarket shelf after a rucksack purchase. It copies a fixed
 * 2-by-3 tile background patch to map tile coordinate (33, 20), then marks the
 * map graphics dirty. It does not change the rucksack level or purchase state;
 * the script calls UpgradeRucksack separately for the actual capacity change.
 * Parameters: none.
 *
 * 购买背包后重绘杂货店货架。它把固定的 2×3 图块背景补丁复制到地图图块坐标
 * (33, 20)，随后标记地图图形需要刷新。它不修改背包等级或购买状态；脚本会
 * 另行调用 UpgradeRucksack 完成真正的容量升级。
 * 参数：无。
 */
void RedrawRucksackShelfAfterPurchase(void);

/*
 * Redraws the supermarket shelf after a Blue Feather purchase. It copies the
 * same fixed 2-by-3 tile background patch to map tile coordinate (33, 23), then
 * marks the map graphics dirty. It neither inserts the Blue Feather nor changes
 * a persistent purchase flag; SetPlayerHeldTool or AddToolToRucksack has already
 * inserted the item before this call.
 * Parameters: none.
 *
 * 购买蓝色羽毛后重绘杂货店货架。它把同一固定的 2×3 图块背景补丁复制到地图
 * 图块坐标 (33, 23)，随后标记地图图形需要刷新。它既不把蓝色羽毛写入背包，
 * 也不修改持久购买标记；调用前 SetPlayerHeldTool 或 AddToolToRucksack 已经
 * 完成物品写入。
 * 参数：无。
 */
void RedrawBlueFeatherShelfAfterPurchase(void);

/*
 * Gets a Harvest Sprite's current assigned task.
 * Parameter: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: HARVEST_SPRITE_TASK_*; HARVEST_SPRITE_TASK_NONE is returned
 * when idle or when the ID does not resolve to a Harvest Sprite.
 *
 * 获取指定小矮人当前承担的工作。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：HARVEST_SPRITE_TASK_*；空闲或 ID 无法解析为小矮人时返回
 * HARVEST_SPRITE_TASK_NONE。
 */
MaryHarvestSpriteTask GetHarvestSpriteCurrentTask(MaryCharacterId sprite_id);

/*
 * Gets the remaining work days for a Harvest Sprite's current assignment.
 * Parameter: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: remaining days, or zero when the ID is invalid.
 *
 * 获取指定小矮人当前委托的剩余工作天数。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：剩余天数；ID 无效时返回零。
 */
MaryHarvestSpriteWorkDays GetHarvestSpriteWorkDaysLeft(MaryCharacterId sprite_id);

/*
 * Gets a Harvest Sprite's experience for one task category.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID; task is
 * HARVEST_SPRITE_TASK_HARVEST, WATER, or ANIMALS.
 * Return value: the stored task experience, or zero when the ID is invalid.
 *
 * 获取指定小矮人在某类工作中的经验值。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID；task 为
 * HARVEST_SPRITE_TASK_HARVEST、WATER 或 ANIMALS。
 * 返回值：保存的工作经验值；ID 无效时返回零。
 */
MaryHarvestSpriteTaskExperience GetHarvestSpriteTaskExperience(
    MaryCharacterId sprite_id,
    MaryHarvestSpriteTask task
);

/*
 * Tests whether a Harvest Sprite has played a minigame today.
 * Parameter: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: nonzero when already played today; zero otherwise.
 *
 * 判断指定小矮人今天是否已经玩过小游戏。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：今天已经玩过时为非零，否则为零。
 */
MaryBool HasHarvestSpritePlayedMinigameToday(MaryCharacterId sprite_id);

/* Tests whether the selected sprite has nonzero minigame experience for a
 * task category. Parameters: sprite_id is the sprite's CHARACTER_* ID; task
 * is HARVEST_SPRITE_TASK_HARVEST, WATER, or ANIMALS. This reads the separate
 * minigame-experience array, not the ordinary task-experience array returned
 * by GetHarvestSpriteTaskExperience(). Return value: nonzero when minigame
 * experience is present for that task; zero for no experience, an invalid
 * task, or an ID that does not resolve to a Harvest Sprite.
 *
 * 判断指定小矮人在某类工作小游戏中是否拥有非零经验。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID；task 为
 * HARVEST_SPRITE_TASK_HARVEST、WATER 或 ANIMALS。本函数读取独立的小游戏
 * 经验数组，并非 GetHarvestSpriteTaskExperience() 返回的普通工作经验数组。
 * 返回值：该类小游戏已有经验时为非零；没有经验、工作类型无效或人物 ID
 * 无法解析为小矮人时返回零。
 */
MaryBool HasHarvestSpriteMinigameExperience(
    MaryCharacterId sprite_id,
    MaryHarvestSpriteTask task
);

/*
 * Assigns work to a Harvest Sprite.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID; task is a working
 * HARVEST_SPRITE_TASK_* value; days is the assignment duration. Vanilla
 * dialogue passes HARVEST_SPRITE_WORK_DAYS_ONE_DAY, THREE_DAYS, or ONE_WEEK.
 *
 * 为指定小矮人安排工作。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID；task 为实际工作用的
 * HARVEST_SPRITE_TASK_*；days 为委托天数。原版雇佣对话传入
 * HARVEST_SPRITE_WORK_DAYS_ONE_DAY、THREE_DAYS 或 ONE_WEEK。
 */
void StartHarvestSpriteTask(
    MaryCharacterId sprite_id,
    MaryHarvestSpriteTask task,
    MaryHarvestSpriteWorkDays days
);

/*
 * Schedules the selected Harvest Sprite's assignment to end at the next daily
 * task update by setting its remaining-work-days field to one. It does not
 * immediately clear the current task. Parameter: sprite_id is the sprite's
 * CHARACTER_* ID; an ID that does not resolve to a Harvest Sprite is a no-op.
 *
 * 把指定小矮人的剩余工作天数字段设为一，使当前委托在下一次每日任务更新时
 * 结束；它不会立即清除当前工作。参数 sprite_id 为该小矮人的 CHARACTER_* ID；
 * 无法解析为小矮人的 ID 不产生操作。
 * 参数：sprite_id 为小矮人的 CHARACTER_* ID。
 */
void ScheduleHarvestSpriteTaskToEndAfterToday(MaryCharacterId sprite_id);

/*
 * Tests whether the selected Harvest Sprite's runtime entity has completed or
 * exhausted the work currently available for the assigned task on the current
 * map. This is a daily/runtime work query, not a test of assignment days left.
 * Parameter: sprite_id is the sprite's CHARACTER_* ID. Return value: nonzero
 * when no currently available work remains; zero when work remains, the ID is
 * outside the seven Harvest Sprite IDs, or the sprite has no entity on the
 * current map.
 *
 * 检查指定小矮人的运行时实体是否已完成当前地图上该工作当天可执行的目标，或
 * 已无可继续处理的目标。这是当日运行时工作查询，不是委托剩余天数查询。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：当天已无可执行工作时为非零；仍有工作、ID 不属于七名小矮人或当前
 * 地图不存在该小矮人实体时为零。
 */
MaryBool IsHarvestSpriteDailyWorkComplete(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's animal-care training minigame and waits
 * for it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: TRUE for the successful/improved result; FALSE otherwise.
 *
 * 运行指定小矮人的动物照料训练小游戏，并等待小游戏结束。成功或能力提升结果
 * 返回 1，未成功结果返回 0。参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：成功或能力提升时为 TRUE，否则为 FALSE。
 */
MaryBool RunHarvestSpriteAnimalCareMinigame(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's harvesting training minigame and waits
 * for it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: TRUE for the successful/improved result; FALSE otherwise.
 *
 * 运行指定小矮人的收获训练小游戏，并等待小游戏结束。成功或能力提升结果返回
 * 1，未成功结果返回 0。参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：成功或能力提升时为 TRUE，否则为 FALSE。
 */
MaryBool RunHarvestSpriteHarvestingMinigame(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's watering training minigame and waits for
 * it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 * Return value: TRUE for the successful/improved result; FALSE otherwise.
 *
 * 运行指定小矮人的浇水训练小游戏，并等待小游戏结束。成功或能力提升结果返回
 * 1，未成功结果返回 0。参数：sprite_id 为该小矮人的 CHARACTER_* ID。
 * 返回值：成功或能力提升时为 TRUE，否则为 FALSE。
 */
MaryBool RunHarvestSpriteWateringMinigame(MaryCharacterId sprite_id);

/*
 * Runs the Chicken Festival contest sequence and waits for its result. Returns
 * 1 when the player's chicken wins and 0 otherwise.
 * Parameters: none.
 * Return value: FESTIVAL_CONTEST_RESULT_WIN for a win and
 * FESTIVAL_CONTEST_RESULT_LOSS otherwise.
 *
 * 运行斗鸡节比赛流程并等待比赛结果。玩家的鸡获胜时返回 1，否则返回 0。
 * 参数：无。
 * 返回值：获胜时为 FESTIVAL_CONTEST_RESULT_WIN，否则为
 * FESTIVAL_CONTEST_RESULT_LOSS。
 */
MaryFestivalContestResult RunChickenFestivalContest(void);

/*
 * Runs the horse-race interface.
 * Parameter: race_mode is FESTIVAL_HORSE_RACE_MODE_*.
 * Return value: -1 when the interface is cancelled, 0 when it exits without a
 * race result, 1 when the player horse wins, or 2 when it loses.
 *
 * 运行赛马界面。参数 race_mode 为 FESTIVAL_HORSE_RACE_MODE_*。
 * 参数：race_mode 为 FESTIVAL_HORSE_RACE_MODE_*。
 * 返回值：取消界面时为
 * -1；未产生比赛结果而退出时为 0；玩家的马获胜时为 1，落败时为 2。
 */
MaryFestivalHorseRaceInterfaceResult RunHorseRace(MaryFestivalHorseRaceMode race_mode);

/*
 * Rebuilds the horse-race entry records, including the randomized NPC horse
 * identifiers and race attributes.
 * Parameter: entry_mode is
 * FESTIVAL_HORSE_RACE_ENTRIES_*; INCLUDE_PLAYER_HORSE marks the player's horse as an
 * entrant, while NPC_ONLY prepares a field without it. The invitation scripts
 * select INCLUDE_PLAYER_HORSE only after the player accepts participation.
 *
 * 重新生成赛马参赛记录，包括随机化的 NPC 马匹编号及比赛属性。
 * 参数：entry_mode 为 FESTIVAL_HORSE_RACE_ENTRIES_*；INCLUDE_PLAYER_HORSE 将玩家的马
 * 标记为参赛，NPC_ONLY 则生成不含玩家马匹的阵容。邀请事件只在玩家同意参赛后
 * 使用 INCLUDE_PLAYER_HORSE。
 */
void PrepareHorseRaceEntries(MaryFestivalHorseRaceEntryMode entry_mode);

/*
 * Opens the horse-race medal exchange interface and waits until it closes.
 * Parameters: none.
 *
 * 打开赛马奖牌兑换界面并等待其关闭。
 * 参数：无。
 */
void OpenHorseRaceMedalExchange(void);

/*
 * Runs the dog-frisbee interface.
 * Parameter: game_mode is FESTIVAL_FRISBEE_MODE_*.
 * Return value is FESTIVAL_CONTEST_RESULT_*; contest scripts store it directly
 * in VAR_FRISBEE_TOURNAMENT_RESULT, while practice scripts may ignore it.
 * Return value: FESTIVAL_CONTEST_RESULT_* for the completed interface result.
 *
 * 运行爱犬飞盘界面。
 * 参数：game_mode 为 FESTIVAL_FRISBEE_MODE_*。
 * 返回值：引擎的飞盘结果 FESTIVAL_CONTEST_RESULT_*；比赛脚本会将其直接写入
 * VAR_FRISBEE_TOURNAMENT_RESULT，自由练习脚本可以忽略该值。
 */
MaryFestivalContestResult RunFrisbeeGame(MaryFestivalFrisbeeMode game_mode);

/* Runs the Frisbee Tournament round entered from the beach-rules sign while
 * the tournament state is active. Normal-day practice uses
 * RunFrisbeeGame(FESTIVAL_FRISBEE_MODE_PRACTICE) instead. The tournament path creates
 * its own modal task in every ROM (MFoMT-US kind 0x28; JP kind 0x27).
 * Parameters: none.
 *
 * 在飞盘大会状态生效时，从海滩规则牌入口运行大会回合。普通日期的练习改走
 * RunFrisbeeGame(FESTIVAL_FRISBEE_MODE_PRACTICE)。四个 ROM 的大会路径都会创建独立的
 * 模态任务（MFoMT-US 类型 0x28，JP 类型 0x27）。
 * 参数：无。
 */
void RunFrisbeeTournamentRound(void);

/*
 * Rebuilds the ten randomized NPC opponent records used by the animal-festival
 * contest interface. The generated records contain unique opponent identifiers
 * and year-scaled contest attributes; no player animal is modified.
 * Parameters: none.
 *
 * 重新生成动物祭典比赛界面使用的十条随机 NPC 对手记录。生成内容包含互不重复
 * 的对手编号及随游戏年份调整的比赛属性，不会修改玩家参赛动物。
 * 参数：无。
 */
void PrepareAnimalFestivalOpponents(void);

/*
 * Returns the item currently selected by the TV Shopping broadcast.
 * Parameters: none.
 * Return value: a TV_SHOPPING_ITEM_* value, or TV_SHOPPING_ITEM_NONE when no
 * item is currently available for ordering.
 *
 * 返回电视购物节目当前选中的商品。
 * 参数：无。
 * 返回值：TV_SHOPPING_ITEM_*；当前没有可订购商品时返回
 * TV_SHOPPING_ITEM_NONE。
 */
MaryTVShoppingItemId GetTVShoppingSelection(void);

/*
 * Returns the item stored as the pending TV Shopping delivery order.
 * Parameters: none.
 * Return value: a TV_SHOPPING_ITEM_* value, or TV_SHOPPING_ITEM_NONE when no
 * delivery is pending.
 *
 * 返回当前等待送货的电视购物商品。
 * 参数：无。
 * 返回值：TV_SHOPPING_ITEM_*；没有等待送货的订单时返回
 * TV_SHOPPING_ITEM_NONE。
 */
MaryTVShoppingItemId GetPendingTVShoppingItem(void);

/*
 * Tests whether the pending TV Shopping order is ready to be delivered.
 * Parameters: none.
 * Return value: 1 when an order exists and its delivery countdown reached
 * zero; otherwise 0.
 * This does not check time of day or competing events. All four vanilla farm
 * entry dispatchers separately require hour >= 6 and < 12 after this test;
 * earlier event branches can take priority over delivery.
 *
 * 检查等待中的电视购物订单是否已经可以送达。
 * 参数：无。
 * 返回值：存在订单且送货倒计时已经归零时为 1，否则为 0。
 * 本函数不检查时段或其他竞争事件。四版原版农场进入分派脚本另行要求小时
 * 大于等于 6 且小于 12；前面的事件分支还可能优先于送货。
 */
MaryBool IsTVShoppingDeliveryReady(void);

/*
 * Selects the item currently offered by the TV Shopping program.
 * Parameter: item_id is TV_SHOPPING_ITEM_* or the exact original item ID.
 * This changes the current selection but does not yet place an order.
 * Clearing this selection to NONE while an order is pending pauses its native
 * countdown: the updater requires both the selection and order to be non-NONE.
 *
 * 选择电视购物节目当前展示的商品。
 * 参数：item_id 为 TV_SHOPPING_ITEM_* 或精确原始商品 ID。
 * 本函数只改变当前选择，尚不会正式下单。
 * 已下单时若把当前选择清为 NONE，会暂停原生送货计数：更新器要求当前选择
 * 和待送货商品均不是 NONE。
 */
void SetTVShoppingSelection(MaryTVShoppingItemId item_id);

/*
 * Confirms the current TV Shopping selection as the pending delivery order.
 * Parameters: none. Call SetTVShoppingSelection first when changing the item.
 * Copies the selection unconditionally and resets the delivery counter to 2.
 * It does not charge money, validate the selection, or preserve an existing
 * pending order; purchase eligibility and payment belong to the caller.
 *
 * 将当前电视购物选择确认为等待送货的订单。
 * 参数：无。需要更换商品时，应先调用 SetTVShoppingSelection。
 * 无条件复制当前选择，并将送货计数重置为 2。不会扣钱、校验所选商品或保留
 * 已有待送货订单；购买资格检查及付款需要由调用方处理。
 */
void ConfirmTVShoppingOrder(void);

/*
 * Completes the pending TV Shopping delivery and clears the stored order.
 * Parameters: none. For farmhouse items, the matching facility or utensil is
 * installed; the Power Berry delivery is finalized after its script-side effect.
 * Cleanup clears the current selection only when it matches the delivered
 * item; a different selection is preserved. The pending item is always set
 * to NONE, but the delivery counter is not reset by the cleanup helper.
 * Utensil installation requires an existing kitchen. If that prerequisite
 * fails, the installer does nothing but the order is still cleared; this
 * callable is not a retryable delivery transaction.
 * Refrigerator, shelf and carpet require a nonzero house upgrade level;
 * the large bed requires a level greater than 1. Kitchen installation also
 * requires a nonzero upgrade level and an existing refrigerator. Failed
 * facility prerequisites likewise do not preserve the pending order.
 *
 * 完成等待中的电视购物送货并清除已保存订单。
 * 参数：无。农舍商品会安装对应设施或厨具；力量果实则在脚本侧效果完成后，
 * 由本函数结束该笔送货。
 * 清理时仅当当前选择与送货商品相同时才清空当前选择；不同的新选择会保留。
 * 待送货商品总会置为 NONE，但清理辅助函数不会重置送货计数。
 * 安装厨具要求已有厨房；不满足时安装器不作修改，但订单仍会清除。
 * 因此本函数不是失败后保留订单、供重试的送货事务。
 * 冰箱、柜子和地毯要求房屋扩建等级非零；大床要求等级大于 1。厨房要求
 * 扩建等级非零且已有冰箱。这些设施的安装条件不满足时，同样不会保留订单。
 */
void CompleteTVShoppingDelivery(void);

/*
 * Tests whether Gotz's Vacation Villa has been built. All four native handlers
 * read bit 1 of the same persistent facility byte whose bit 0 is set by
 * BuildMountainCottage and whose bit 2 is set by BuildSeasideCottage. FoMT
 * stores that account/facility record at save offset 0x1AA8 and MFoMT at
 * 0x1AB8. This reads the completed persistent unlock, not a pending order or
 * construction timer.
 * Parameters: none.
 * Return value: nonzero after Gotz's Vacation Villa construction completes;
 * zero otherwise.
 *
 * 判断 Gotz 建造的 Vacation Villa 是否已经建成。四版原生 handler 均读取同一
 * 持久设施字节的 bit 1；该字节的 bit 0 由 BuildMountainCottage 设置，bit 2
 * 由 BuildSeasideCottage 设置。该账户／设施记录在 FoMT 存档中的偏移为
 * 0x1AA8，在 MFoMT 中为 0x1AB8。本函数读取的是已经完成的持久解锁状态，
 * 不是等待中的订单或施工倒计时。
 * 参数：无。
 * 返回值：Gotz 完成别墅施工后为非零，否则为零。
 */
MaryBool IsVacationVillaBuilt(void);

/*
 * Tests the maximum-friendship completion condition for every required,
 * currently present villager. The native loop covers character IDs 1 through
 * 42, statically excludes Cliff, Kai, Gourmet, Kappa, and the player's child,
 * skips absent NPC records, and requires friendship greater than 249 for every
 * remaining record. It tests current friendship, not a recorded milestone.
 * Parameters: none.
 * Return value: nonzero when the condition is satisfied; zero otherwise.
 *
 * 判断所有要求计入且当前存在的村民是否满足最高友好度完成条件。原生循环覆盖
 * 人物 ID 1 至 42，固定排除 Cliff、Kai、Gourmet、Kappa 及玩家孩子，跳过当前
 * 不存在的 NPC 记录，并要求其余每条记录的友好度均大于 249。检查的是当前
 * 友好度，不是已经登记过的里程碑。
 * 参数：无。
 * 返回值：满足条件时为非零，否则为零。
 */
MaryBool AreAllRequiredVillagersAtMaxFriendship(void);

/*
 * Tests all 15 crop-product records. Every record must be visible in the
 * shipping list and have a nonzero shipped count; this reads current records,
 * not a separately latched achievement flag.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 检查全部 15 条作物出货记录。每条记录都必须已在出货表中显示且出货数量
 * 非零；读取的是当前记录，不是另行锁存的成就标志。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasShippedOneOfEachCrop(void);

/*
 * Tests the maximum-affection condition for the dog, the horse when present,
 * and every present chicken, cow, and sheep in the active coop/barn capacity.
 * Empty animal slots are skipped; every present animal must have affection
 * greater than 199. This reads current affection, not a recorded milestone.
 * Parameters: none.
 * Return value: nonzero when the condition is satisfied; zero otherwise.
 *
 * 检查狗、当前存在的马，以及鸡舍／畜棚有效容量内所有实际存在的鸡、牛、羊
 * 是否满足最高好感度条件。空动物槽会跳过；所有存在动物的好感度必须大于
 * 199。读取的是当前好感度，不是已经登记的里程碑。
 * 参数：无。
 * 返回值：满足条件时为非零，否则为零。
 */
MaryBool AreAllFarmAnimalsAtMaxAffection(void);

/*
 * Tests all 20 mineral-product records. Every record must be visible in the
 * shipping list and have a nonzero shipped count.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 检查全部 20 条矿物出货记录；每条记录都必须已在出货表中显示且出货数量
 * 非零。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasShippedOneOfEachMineral(void);

/*
 * Tests fishing-record slots 8 through 58: all 45 ordinary fish and all six
 * Fish Kings. The eight preceding non-fish catches are intentionally excluded;
 * every checked record must have a nonzero caught count.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 检查钓鱼纪录槽 8 至 58，即全部 45 种普通鱼和六种鱼王。前八条非鱼类钓获
 * 物会明确排除；每条被检查记录的捕获数都必须非零。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasCaughtEveryFishSpecies(void);

/*
 * Sums the caught-count field of fishing-record slots 8 through 58: all 45
 * ordinary fish and all six Fish Kings. The eight preceding non-fish catches
 * are excluded. The running total saturates at 1,000,000,000.
 * Parameters: none.
 * Return value: the saturated cumulative fish count.
 *
 * 累加钓鱼纪录槽 8 至 58 的捕获数，即全部 45 种普通鱼和六种鱼王；前八条
 * 非鱼类钓获物不计入。累计值最高饱和为 1,000,000,000。
 * 参数：无。
 * 返回值：经过上述饱和处理的累计捕获鱼数量。
 */
MaryFishCount GetTotalFishCaught(void);

/*
 * Tests the persistent bit set when CollectBlacksmithOrder successfully places
 * a completed Mythic sickle, hoe, axe, hammer, watering can, or fishing rod
 * order. The vanilla scripts use it as the prerequisite for recording the
 * GameCube-link Mythic Tool milestone.
 * Parameters: none.
 * Return value: nonzero after at least one of those six orders has been
 * collected successfully; zero otherwise. Merely possessing an injected tool
 * or failing collection for lack of space does not establish this bit.
 *
 * 检查持久状态位：CollectBlacksmithOrder 成功放入已经完成的贤者镰刀、锄头、
 * 斧、锤、洒水壶或钓竿订单成品时置位。原版脚本将它用作记录 GameCube 联机
 * “贤者农具”里程碑的前置条件。
 * 参数：无。
 * 返回值：上述六种订单至少成功领取过一种后为非零，否则为零。仅通过其他方式
 * 把农具放入存档，或因空间不足导致领取失败，不能据此证明该位会置位。
 */
MaryBool HasObtainedMythicTool(void);

/* Tests every one of the 103 physical shipping-product records, IDs 0x00
 * through 0x66. Each record must be visible in the shipping list and have a
 * nonzero shipped count. This is broader than the crop and mineral subsets.
 * Parameters: none.
 * Return value: TRUE when all 103 products meet both conditions; FALSE
 * otherwise.
 *
 * 检查全部 103 条物理出货品记录（ID 0x00 至 0x66）。每条记录都必须已在
 * 出货表中显示且出货数量非零；该范围比作物和矿物子集更广。
 * 参数：无。
 * 返回值：全部 103 项同时满足两个条件时为 TRUE，否则为 FALSE。
 */
MaryBool HasShippedOneOfEachProduct(void);

/*
 * Returns the player's current money. The four native handlers directly load
 * the saved 32-bit balance (FoMT save offset 0x1AA8; MFoMT 0x1AB8) and push it
 * to the VM without clamping or repairing it. Normal AddMoney writes cap the
 * balance at 1,000,000,000 G, so an ordinary save remains in the positive
 * MaryMoneyBalance range; externally corrupted high-bit values retain their
 * exact bits and are not part of the supported semantic domain.
 * Parameters: none.
 * Return value: current game-currency amount.
 *
 * 返回玩家当前金钱。四版原生 handler 都直接读取存档中的 32 位余额（FoMT
 * 偏移 0x1AA8，MFoMT 偏移 0x1AB8）并压入 VM，不执行截断或修复。正常的
 * AddMoney 写入会把余额封顶为 1,000,000,000 G，因此正常存档始终落在正数
 * MaryMoneyBalance 范围；外部破坏造成的高位值会保留原始比特，不属于受支持的
 * 语义取值域。
 * 参数：无。
 * 返回值：当前游戏货币数量。
 */
MaryMoneyBalance GetMoney(void);

/*
 * Adds game currency to the player.
 * Parameter: amount is the currency amount to add.
 * Native arithmetic treats amount as unsigned and caps a valid balance at
 * 1,000,000,000 G; do not use a negative amount to subtract money. The routine
 * also updates account records. In MFoMT-US/JP, reaching the cap sets the
 * boolean read by VAR_SAVED_ONE_BILLION_G_ACHIEVEMENT_RECORDED; FoMT-US/JP
 * do not perform this extra flag write.
 *
 * 增加玩家的游戏货币。
 * 参数：amount 为增加的货币数量。
 * 原生运算把 amount 视为无符号数，并将有效余额封顶为 1,000,000,000 G；
 * 不要用负数表示扣款。函数还会更新账户记录。女孩美版／日版达到上限时，
 * 会设置 VAR_SAVED_ONE_BILLION_G_ACHIEVEMENT_RECORDED 所读取的布尔标志；
 * 男孩美版／日版没有这一步额外标志写入。
 */
void AddMoney(MaryMoneyAmount amount);

/*
 * Subtracts game currency from the player.
 * Parameter: amount is the currency amount to subtract.
 * The native amount comparison is unsigned. Insufficient funds leave the
 * balance unchanged, rather than clamping it to zero. Empty account records
 * may still be initialized before this check. The native success/failure
 * result is discarded by the script dispatcher: this callable returns void.
 * Check GetMoney() before a purchase; do not use negative amounts as refunds.
 *
 * 扣除玩家的游戏货币。
 * 参数：amount 为扣除的货币数量。
 * 原生金额比较按无符号数进行。余额不足时不修改余额，而不是扣到零；
 * 但在检查前仍可能初始化空的账户记录。底层成功／失败返回值被脚本派发器
 * 丢弃，因此此 callable 返回 void。购买前应检查 GetMoney()，不要用负数退款。
 */
void SubtractMoney(MaryMoneyAmount amount);

/*
 * Enables the global scripted-NPC-control mode used while a coordinated event
 * owns participant movement and per-NPC event scripts. In this mode the normal
 * NPC schedule/update path observes the event-control flag instead of freely
 * advancing participants. Pair it with DisableScriptedNpcControl when the
 * event's participant scripts have finished.
 * This flag also suppresses the player entity's stamina/fatigue adjustment
 * path, including ChangePlayerStaminaAndFatigue. It is not NPC-only.
 * Audited FoMT-US and MFoMT-JP tool-action paths also skip experience awards
 * while this mode is active. This is a guard on those action paths, not a
 * blanket prohibition on direct writes to tool-experience records.
 * Parameters: none.
 * Return value: none.
 *
 * 启用全局的脚本化 NPC 控制模式，用于由事件统一控制参与者移动和各 NPC
 * 事件脚本的场景。启用后，常规 NPC 日程/更新流程会遵循事件控制标志，
 * 不再自行推进参与者。参与者脚本结束后应调用 DisableScriptedNpcControl。
 * 此标志还会屏蔽玩家实体的体力／疲劳调整路径，包括
 * ChangePlayerStaminaAndFatigue，因此其影响不只限于 NPC。
 * 已核对的男孩US、女孩JP农具动作路径还会在该模式下跳过经验发放。
 * 这是动作路径上的判断，不代表所有直接写入农具经验记录的操作都被禁止。
 * 参数：无。
 * 返回值：无。
 */
void EnableScriptedNpcControl(void);

/*
 * Disables scripted-NPC-control mode and returns NPC updates to the normal
 * schedule path. Event cleanup scripts call this before removing or restoring
 * their participant entities.
 * This only clears the shared mode byte. It does not replay stamina/fatigue
 * changes that were skipped while the mode was enabled.
 * Parameters: none.
 * Return value: none.
 *
 * 关闭脚本化 NPC 控制模式，使 NPC 更新恢复到常规日程流程。事件清理脚本会在
 * 移除或恢复参与者实体之前调用它。
 * 本函数仅清除共享模式字节，不会补执行此前启用期间被跳过的体力／疲劳调整。
 * 参数：无。
 * 返回值：无。
 */
void DisableScriptedNpcControl(void);

/*
 * Returns bits 0-5 of the current blacksmith order record.
 * Parameters: none.
 * Return value: a BLACKSMITH_ORDER_* value; BLACKSMITH_ORDER_NONE means no
 * order is pending. The vanilla order domain is 0-37 even though the physical
 * field is six bits wide.
 *
 * 返回当前锻冶屋订单记录的 bit 0-5。
 * 参数：无。
 * 返回值：BLACKSMITH_ORDER_*；BLACKSMITH_ORDER_NONE 表示当前没有订单。物理
 * 字段宽六位，但原版订单域只使用 0-37。
 */
MaryBlacksmithOrderId GetBlacksmithOrderId(void);

/*
 * Tests whether an order exists and its three-bit remaining-days field is zero.
 * Parameters: none.
 * Return value: nonzero only when order bits 0-5 are nonzero and timer bits
 * 6-8 are all zero; zero when no order exists or at least one timer bit remains.
 *
 * 检查当前是否存在订单，并且其三位剩余天数字段是否已经归零。
 * 参数：无。
 * 返回值：仅当订单 bit 0-5 非零且计时 bit 6-8 全为零时为非零；没有订单或
 * 计时字段仍非零时返回零。
 */
MaryBool IsBlacksmithOrderReady(void);

/*
 * Tries to collect the completed blacksmith order. This operation places the
 * result in the player's held slot, rucksack, or appropriate storage and
 * clears the pending order when placement succeeds. Successfully collecting
 * any of the six Mythic tool orders also sets the persistent bit returned by
 * HasObtainedMythicTool().
 * Parameters: none. Call only after IsBlacksmithOrderReady() succeeds.
 * Return value: a BLACKSMITH_COLLECTION_* placement result. NO_SPACE leaves
 * the order pending so collection can be retried.
 *
 * 尝试领取已经完成的锻冶屋订单。该操作会把成品放入玩家手持栏、背包或对应
 * 仓库；成功放入后会清除等待中的订单。成功领取六种贤者农具订单中的任意一种，
 * 还会设置 HasObtainedMythicTool() 所返回的持久状态位。
 * 参数：无。应在 IsBlacksmithOrderReady() 成功后调用。
 * 返回值：BLACKSMITH_COLLECTION_* 放置结果。NO_SPACE 不会清除订单，可在
 * 腾出空间后再次领取。
 */
MaryBlacksmithCollectionResult CollectBlacksmithOrder(void);

/*
 * Requests the target in-game clock time.
 * Parameters: hour is the 0-23 hour; minute is the 0-59 minute. The engine
 * packs them into a five-bit hour and six-bit minute target.
 * These ranges are conventional clock values, not native clamps: hour is
 * masked with 31 and minute with 63, with no carry normalization. Values
 * outside 0-23/0-59 therefore need not produce a valid clock time.
 * All four targets advance the active map simulation until the time is reached,
 * including midnight date rollover, instead of simply overwriting the clock.
 * It advances before comparing, so an equal target is not a no-op. Invalid
 * target times can be unreachable; do not use them.
 *
 * 请求将游戏时钟推进至目标时间。
 * 参数：hour 为 0-23 时，minute 为 0-59 分。目标值分别打包为 5 位小时字段和
 * 6 位分钟字段。
 * 上述范围是常规时钟取值，不是原生限幅规则：小时按 31 掩码、分钟按 63
 * 掩码截断，不进行进位归一化。超出 0-23／0-59 的值不保证形成有效时间。
 * 四个版本都会推进当前地图模拟直到目标时刻，包括跨午夜更新日期，而不是直接
 * 覆盖时钟。先推进后比较，所以目标等于当前时间也不是空操作。非法目标可能
 * 无法到达，不应使用。
 */
void SetGameTime(MaryClockHour hour, MaryClockMinute minute);

/*
 * Tests the pending/unread bitmap for one mailbox letter. The native state is
 * indexed by letter_id - 77: FoMT accepts IDs 77-136 and MFoMT IDs 77-189.
 * Parameter: letter_id is a LETTER_* value for the selected target.
 * Return value: TRUE while the letter is pending/unread; FALSE otherwise.
 *
 * 检查指定邮箱信件的待阅读位图。原生状态以 letter_id - 77 为索引：FoMT
 * 接受 ID 77-136，MFoMT 接受 ID 77-189。
 * 参数：letter_id 为所选目标的 LETTER_* 值。
 * 返回值：信件尚待阅读时为 TRUE，否则为 FALSE。
 */
MaryBool IsLetterWaiting(MaryLetterId letter_id);

/*
 * Tests the read/archive bitmap for one mailbox letter. This does not include
 * a newly delivered letter that is still pending/unread.
 * Parameter: letter_id is a LETTER_* value for the selected target.
 * Return value: TRUE after the letter has been marked read; FALSE otherwise.
 *
 * 检查指定邮箱信件的已读／归档位图；刚投递但仍待阅读的信件不属于此状态。
 * 参数：letter_id 为所选目标的 LETTER_* 值。
 * 返回值：信件已标记为已读时为 TRUE，否则为 FALSE。
 */
MaryBool HasReceivedLetter(MaryLetterId letter_id);

/*
 * Delivers a letter by setting its pending/unread bit. Repeating the call is
 * idempotent because the native operation only sets the same bit.
 * Parameter: letter_id is a LETTER_* value for the selected target.
 *
 * 通过设置待阅读位来投递信件。重复调用只会再次设置同一位，因此结果幂等。
 * 参数：letter_id 为所选目标的 LETTER_* 值。
 */
void DeliverLetter(MaryLetterId letter_id);

/*
 * Marks a letter as read by clearing its pending/unread bit and setting its
 * read/archive bit. Repeating the operation leaves the same final state.
 * Parameter: letter_id is a LETTER_* value for the selected target.
 *
 * 将信件标记为已读：清除待阅读位，并设置已读／归档位。重复操作保持相同终态。
 * 参数：letter_id 为所选目标的 LETTER_* 值。
 */
void MarkLetterRead(MaryLetterId letter_id);

/*
 * Returns the number of set bits in the pending/unread letter map.
 * Parameters: none.
 * Return value: 0-60 in FoMT or 0-113 in MFoMT.
 *
 * 返回待阅读信件位图中已设置位的数量。
 * 参数：无。
 * 返回值：FoMT 为 0-60，MFoMT 为 0-113。
 */
MaryLetterCount GetWaitingLetterCount(void);

/*
 * Returns the number of set bits in the read/archive letter map.
 * Parameters: none.
 * Return value: 0-60 in FoMT or 0-113 in MFoMT.
 *
 * 返回已读／归档信件位图中已设置位的数量。
 * 参数：无。
 * 返回值：FoMT 为 0-60，MFoMT 为 0-113。
 */
MaryLetterCount GetSavedLetterCount(void);

/*
 * Displays one television-program text through the television viewer and waits
 * for its navigation action. Unlike TalkMessage, this operation returns the
 * viewer action used by the caller to change channels or leave the program.
 * Parameters: message is a string from the current script's mary_text_table.
 * Return value: a MaryTelevisionInput value. Directional results select a
 * channel; TELEVISION_INPUT_ADVANCE_TEXT continues the current text and
 * TELEVISION_INPUT_TURN_OFF leaves the television.
 * Call SetTelevisionProgram first when the program requires a specific visual
 * presentation, then call EndTelevisionProgram after the last message.
 * All four native leaves submit the text with rate 0x40 and request viewer
 * state 0x11, rather than using the ordinary dialogue-window state.
 *
 * 通过电视观看界面显示一段节目文本，并等待导航操作。它与 TalkMessage 不同，
 * 会返回供调用脚本切换频道或退出节目使用的观看界面操作值。
 * 参数：message 为当前脚本 mary_text_table 中的字符串。
 * 返回值：MaryTelevisionInput。方向键结果用于选择频道；
 * TELEVISION_INPUT_ADVANCE_TEXT 继续当前文本，TELEVISION_INPUT_TURN_OFF
 * 离开电视界面。
 * 需要特定节目画面时先调用 SetTelevisionProgram，最后一段文本后调用
 * EndTelevisionProgram。
 * 四版原生叶函数均以速率 0x40 提交文本并请求观看器状态 0x11，不使用普通
 * 对话窗口状态。
 */
MaryTelevisionInput ShowTelevisionMessage(const char *message);

/*
 * Selects the television program presentation used by subsequent television
 * messages. The value is a MaryTelevisionProgramId, not a script ID. All four
 * native leaves dispatch through television-viewer virtual method offset 0x118.
 * Parameters: program_id is the television program/presentation slot.
 * Return value: none.
 *
 * 选择后续电视文本使用的节目显示样式。该值是 MaryTelevisionProgramId，
 * 不是脚本 ID。四版原生叶函数均通过电视观看器虚表偏移 0x118 分派。
 * 参数：program_id 为电视节目/显示槽位。
 * 返回值：无。
 */
void SetTelevisionProgram(MaryTelevisionProgramId program_id);

/*
 * Ends the active television-program presentation after its final message.
 * All four native leaves dispatch the no-argument operation through the
 * adjacent television-viewer virtual method offset 0x11C.
 * Parameters: none.
 * Return value: none.
 *
 * 在最后一段节目文本完成后结束当前电视节目显示。四版原生叶函数均通过相邻的
 * 电视观看器虚表偏移 0x11C 分派这一无参数操作。
 * 参数：无。
 * 返回值：无。
 */
void EndTelevisionProgram(void);

/*
 * Re-evaluates the current schedule and path of every live NPC slot. The engine
 * iterates NPC IDs 1 through 35 and invokes each NPC entity's schedule refresh
 * in mode 3, which selects the appropriate schedule state for the current game
 * context. Event cleanup commonly calls this after scripted participants leave.
 * Parameters: none.
 * Return value: none.
 *
 * 重新计算所有现存 NPC 槽位当前应使用的日程与路径。引擎遍历 NPC ID 1～35，
 * 并以模式 3 调用各 NPC 实体的日程刷新逻辑，从当前游戏状态重新选择日程。
 * 事件参与者离场后，清理脚本通常会调用此函数。
 * 参数：无。
 * 返回值：无。
 */
void RefreshAllNpcSchedules(void);

/*
 * Tests whether an animal kind/index pair resolves to a currently registered
 * animal record. This is a resolver-presence check, not a separate life-state
 * flag test. Cow, sheep and chicken use guarded occupied-roster lookups; horse
 * and dog ignore animal_index. Kinds outside ANIMAL_KIND_HORSE..ANIMAL_KIND_DOG
 * resolve to null.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: TRUE when the resolver returns a record; FALSE otherwise.
 * Call this before indexed animal getters while iterating barn or coop slots.
 *
 * 判断动物类别与槽位组合是否能解析到当前已登记的动物记录。这是解析器的
 * “记录存在”检查，不会另外查询生命状态标志。牛、羊、鸡使用带占用状态和
 * 种类检查的名册 getter；马和狗忽略 animal_index。超出
 * ANIMAL_KIND_HORSE～ANIMAL_KIND_DOG 的类别会解析为空。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该动物类别内从 0 开始的槽位。
 * 返回值：解析器返回记录时为 TRUE，否则为 FALSE。
 * 遍历牛羊棚或鸡舍槽位时，应先调用本函数再读取动物属性。
 */
MaryBool DoesAnimalExist(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Creates and registers the farm's horse, then creates its runtime entity.
 * Parameters: skip_creation must be FALSE to perform creation; any nonzero
 * value returns without creating the horse. The reachable initial facing is
 * therefore always zero (down), not a selectable direction. age_stage is a
 * MaryAnimalHorseAgeStage whose value is converted to age_stage * 120 days and
 * stored in a ten-bit age field. Noncanonical values wrap rather than clamp
 * to a growth stage (for example, 128 becomes zero days);
 * map_id is the initial MaryMapId; x and y are absolute map coordinates.
 * The shipped horse-offer events use entity slot 44 internally after this
 * call.
 * If the farm already has a registered horse, the registration helper keeps
 * its existing record. The outer routine still proceeds to runtime-entity
 * setup; this is not a general overwrite operation or a guaranteed no-op.
 *
 * 创建并登记农场的马，随后创建其运行时实体。
 * 参数：skip_creation 必须为 FALSE 才执行创建，任何非零值都会直接返回、不创建马。
 * 因此实际可达的初始朝向固定为零（向下），不是可选朝向。age_stage 为
 * MaryAnimalHorseAgeStage，引擎会将其
 * 换算为 age_stage * 120 天后存入 10 位年龄字段。非标准值会截断回绕，
 * 而非限制到有效成长阶段（例如 128 会变成零天）。
 * map_id 为初始 MaryMapId；x、y 为绝对地图坐标。
 * 原版送马事件在本调用后使用运行时实体槽 44。
 * 如果农场已有登记的马，登记辅助函数保留原记录；外层函数仍继续运行时实体
 * 的设置流程。因此这既不是通用覆盖操作，也不能保证重复调用完全无影响。
 */
void CreateFarmHorse(
    MaryBool skip_creation,
    MaryAnimalHorseAgeStage age_stage,
    MaryMapId map_id,
    MaryMapSpaceX x,
    MaryMapSpaceY y);

/*
 * Removes runtime horse entity 44 and clears the registered farm horse when
 * skip_removal is FALSE. A nonzero skip_removal makes the native routine return
 * without changing either object; it is a guard value, not a family of removal
 * modes. The VM consumes unused_value, but the native routine does
 * not use its value: on the removal path its incoming register is overwritten
 * by the entity-manager method pointer before being read. All shipped calls
 * pass zero for both arguments. The native entry code agrees in all four ROMs.
 * The farm-data step clears only the horse-present flag; it does not erase
 * the stored horse record. That record becomes inaccessible through GetHorse
 * while the flag is clear. Do not interpret removal as zeroing all horse data.
 * Parameters: skip_removal performs removal only when FALSE; unused_value is
 * popped by the VM but never read by the native operation.
 *
 * 当 skip_removal 为 FALSE 时，删除运行时马实体 44，并清除农场数据中登记的马；
 * 非零值会让原生函数直接返回，不改变两处对象，所以它是移除保护值而不是多种
 * “移除模式”。unused_value 会被 VM 从栈中取出，但原生函数不使用其值：在实际移除
 * 路径上，保存该值的传入寄存器在读取前被实体管理器的方法指针覆盖。原版脚本
 * 的两个参数均为零。四个 ROM 的原生入口代码对此一致。
 * 农场数据处理只清除“有马”标志，不擦除保存的马匹记录；标志清除后 GetHorse
 * 不再返回该记录。不要把移除理解为将全部马匹数据清零。
 * 参数：skip_removal 为 FALSE 时执行移除；unused_value 只保留被 VM 弹出的原始
 * 栈操作数，原生逻辑不读取其值。
 */
void RemoveFarmHorse(MaryBool skip_removal, int unused_value);

/*
 * Copies the selected animal's name into a text string-substitution slot.
 * Parameters: text_variable is TEXT_VARIABLE_1 through TEXT_VARIABLE_4;
 * animal_kind is ANIMAL_KIND_*; animal_index follows GetAnimalAffection's
 * shared barn/separate coop roster rules (horse/dog ignore the index).
 * Use DoesAnimalExist before calling this with a dynamically iterated index.
 * If animal resolution fails or the text context is absent, the handler
 * returns without updating or clearing the substitution slot. An earlier
 * string can therefore remain; do not treat this as an empty-string setter.
 * The native substitution copier truncates by bytes: FoMT-US 22, FoMT-JP 20,
 * MFoMT-US 28, MFoMT-JP 20, then adds a terminator. These are copier limits,
 * not animal-name input limits or Unicode character counts.
 * The native copier does not bounds-check text_variable before computing
 * the destination address. Use only the four declared TEXT_VARIABLE_* slots;
 * an out-of-range integer is not a supported extra substitution slot.
 *
 * 将指定动物名称复制到文本字符串替换槽。
 * 参数：text_variable 为 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4；animal_kind 为
 * ANIMAL_KIND_*；animal_index 遵循 GetAnimalAffection 的牛羊共享畜棚、鸡另用
 * 鸡舍名册规则（马和狗忽略槽位）。
 * 动态遍历槽位时，应先使用 DoesAnimalExist 检查。
 * 无法解析到动物或文本上下文不存在时，handler 直接返回，不更新也不清空
 * 替换槽。因此旧字符串可能仍然保留，不能把此调用当成设置空字符串。
 * 原生替换复制器按字节截断：FoMT-US 为 22、FoMT-JP 为 20、MFoMT-US 为 28、
 * MFoMT-JP 为 20，之后添加终止字节。这是复制器上限，不是动物命名输入上限，
 * 也不是 Unicode 字符数。
 * 原生复制器在计算目标地址前不检查 text_variable 边界。仅使用声明的四个
 * TEXT_VARIABLE_* 槽；越界整数不是可用的额外替换槽。
 */
void GetAnimalName(
    MaryTextVariableSlot text_variable,
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index
);

/*
 * Returns the current animal-target selector stored by scene entity 0 (the
 * player entity). The native helper looks up entity 0 and calls its virtual
 * getter at vtable offset 0x78; it does not read the script event record and
 * does not validate or clamp the result. Vanilla calls it only from animal
 * interaction events, where the value is a zero-based livestock roster slot.
 * Parameters: none. The script determines the animal family separately, then
 * uses this selector with typed animal getters and setters. It is not a global
 * entity ID. Outside a prepared animal interaction its meaning is unsupported.
 * Return value: the zero-based roster slot selected by the prepared animal
 * interaction; outside that context the raw value has no supported meaning.
 *
 * 返回场景实体 0（玩家实体）保存的当前动物目标选择值。原生辅助函数固定查找
 * 实体 0，再调用其虚表偏移 0x78 的 getter；它不读取脚本事件记录，也不验证或
 * 限制返回值。原版仅在动物交互事件中调用，此时该值是从 0 开始的家畜名册槽。
 * 参数：无。脚本会另外确定动物类别，再将此选择值传给带类型的动物属性读写
 * 函数。该值不是全局实体 ID；脱离已准备好的动物交互时，其含义不受支持。
 * 返回值：已准备动物交互所选的从零开始名册槽；脱离该上下文时，原始值没有
 * 受支持的含义。
 */
MaryAnimalSlotIndex GetInteractingAnimalIndex(void);

/*
 * Tests the selected animal's daily talked-to flag.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * roster slot (cows/sheep share the barn roster; chickens use the coop roster;
 * horse and dog ignore the index). Failed animal resolution returns zero.
 * Return value: the normalized one-bit daily flag at record byte 0x19 bit 4.
 *
 * 判断指定动物当日是否已经交谈。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为从 0 开始的名册槽位，
 * 牛羊共享畜棚名册，鸡另用鸡舍名册，马和狗忽略槽位。动物解析失败时返回零。
 * 返回值：记录字节 0x19 第 4 位的规范化布尔值；当日已经交谈时为 TRUE。
 */
MaryBool HasAnimalBeenTalkedTo(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Sets the selected animal's daily talked-to flag.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * roster slot (cows/sheep share the barn roster; chickens use the coop roster).
 * Pair this with HasAnimalBeenTalkedTo around the completed interaction.
 * Idempotent: only sets the talked bit; it does not open dialogue or increase
 * affection. Vanilla interaction scripts call AddAnimalAffection separately.
 * Failed animal resolution performs no write; horse/dog ignore the index. The
 * native setter sets record byte 0x19 bit 4 and preserves every other bit.
 *
 * 设置指定动物的当日已交谈标志。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为从 0 开始的名册槽位，
 * 牛羊共享畜棚名册，鸡另用鸡舍名册。
 * 应与 HasAnimalBeenTalkedTo 配合，在交互完成时设置。
 * 重复调用不会重复产生效果：只设置已交谈位，不打开对话，也不增加好感度。
 * 原版交互脚本另外调用 AddAnimalAffection。动物解析失败时不写入；马和狗
 * 忽略槽位参数。原生 setter 只设置记录字节 0x19 的第 4 位并保留其余位。
 */
void SetAnimalTalkedTo(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Adds a signed amount to the selected animal's affection.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index follows the shared
 * barn/separate coop roster rules documented by GetAnimalAffection (horse
 * and dog ignore it); amount is the signed affection change.
 * The native 32-bit sum is limited to 0-250 after addition, not the NPC
 * friendship limit of 255. Avoid extreme amounts that overflow before the
 * checks. If animal resolution returns null, no affection write occurs.
 *
 * 为指定动物增加有符号好感度变化量。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 遵循 GetAnimalAffection
 * 注明的牛羊共享畜棚、鸡另用鸡舍的名册规则（马和狗忽略此参数）。
 * amount 为有符号好感度变化量。
 * 原生先进行 32 位加法，再将结果限制到 0-250，并非 NPC 友好度的 255。
 * 应避免在检查前导致溢出的极端增量。无法解析到动物时不写入好感度。
 */
void AddAnimalAffection(
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index,
    MaryAnimalAffectionDelta amount
);

/*
 * Tests whether the selected livestock animal is unhappy.
 * Use only cow, sheep or chicken kinds; horse/dog are not Livestock.
 * The handler checks null, not the resolved object's class. Unsupported kinds
 * read an unrelated field rather than reliably returning zero; failed animal
 * resolution returns zero.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * roster slot (cows/sheep share the barn roster; chickens use the coop roster).
 * Return value: nonzero when unhappy; zero otherwise.
 * The exact stored field is record byte 0x1C bit 6.
 *
 * 判断指定家畜是否处于不高兴状态。
 * 仅用于牛、羊或鸡；马和狗不是 Livestock 类型。
 * handler 仅检查空指针，不验证解析对象的类别。不支持的种类会读取无关字段，
 * 不能保证返回零；无法解析到动物时才直接返回零。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为从 0 开始的名册槽位，
 * 牛羊共享畜棚名册，鸡另用鸡舍名册。
 * 返回值：不高兴时为非零，否则为零。
 * 对应的准确存储字段为记录字节 0x1C 第 6 位。
 */
MaryBool IsAnimalUnhappy(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected livestock animal is sick.
 * Use only cow, sheep or chicken kinds; horse/dog are not Livestock.
 * The handler checks null, not the resolved object's class. Unsupported kinds
 * read an unrelated field rather than reliably returning zero; failed animal
 * resolution returns zero.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * roster slot (cows/sheep share the barn roster; chickens use the coop roster).
 * Return value: nonzero when sick; zero otherwise.
 * The exact stored field is record byte 0x1C bit 7.
 *
 * 判断指定家畜是否生病。
 * 仅用于牛、羊或鸡；马和狗不是 Livestock 类型。
 * handler 仅检查空指针，不验证解析对象的类别。不支持的种类会读取无关字段，
 * 不能保证返回零；无法解析到动物时才直接返回零。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为从 0 开始的名册槽位，
 * 牛羊共享畜棚名册，鸡另用鸡舍名册。
 * 返回值：生病时为非零，否则为零。
 * 对应的准确存储字段为记录字节 0x1C 第 7 位。
 */
MaryBool IsAnimalSick(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected barn animal is pregnant.
 * Use only cow or sheep kinds; chicken/horse/dog are not BarnAnimal.
 * The handler checks null, not the resolved object's class. Unsupported kinds
 * read an unrelated field rather than reliably returning zero; failed animal
 * resolution returns zero.
 * Parameters: animal_kind is ANIMAL_KIND_COW or ANIMAL_KIND_SHEEP;
 * animal_index is the zero-based shared barn-roster slot.
 * Return value: record byte 0x24 bit 0, normalized to zero or one.
 *
 * 判断指定牛羊棚动物是否怀孕。
 * 仅用于牛或羊；鸡、马、狗不是 BarnAnimal 类型。
 * handler 仅检查空指针，不验证解析对象的类别。不支持的种类会读取无关字段，
 * 不能保证返回零；无法解析到动物时才直接返回零。
 * 参数：animal_kind 为 ANIMAL_KIND_COW 或 ANIMAL_KIND_SHEEP；animal_index
 * 为共享牛羊棚中从 0 开始的槽位。
 * 返回值：记录字节 0x24 第 0 位，规范化为零或一。
 */
MaryBool IsAnimalPregnant(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Returns the healthy-pregnancy day counter for the selected barn animal.
 * Use only cow or sheep kinds; chicken/horse/dog are not BarnAnimal.
 * The handler checks null, not the resolved object's class. Unsupported kinds
 * read an unrelated field rather than reliably returning zero; failed animal
 * resolution returns zero.
 * Parameters: animal_kind is ANIMAL_KIND_COW or ANIMAL_KIND_SHEEP;
 * animal_index is the zero-based shared barn-roster slot.
 * Return value: the five-bit healthy-pregnancy counter (0-31), or zero when
 * not pregnant. This is elapsed healthy pregnancy time, not remaining days:
 * vanilla cow/sheep interaction scripts subtract it from 21 for dialogue.
 * Treat it as a numeric count, not an event-state enum.
 *
 * 返回指定牛羊棚动物的健康怀孕天数计数。
 * 仅用于牛或羊；鸡、马、狗不是 BarnAnimal 类型。
 * handler 仅检查空指针，不验证解析对象的类别。不支持的种类会读取无关字段，
 * 不能保证返回零；无法解析到动物时才直接返回零。
 * 参数：animal_kind 为 ANIMAL_KIND_COW 或 ANIMAL_KIND_SHEEP；animal_index
 * 为共享牛羊棚中从 0 开始的槽位。
 * 返回值：五位健康怀孕天数计数（0-31）；未怀孕时为零。这是已累计的健康
 * 怀孕天数，不是剩余天数；原版牛羊交互脚本用 21 减去它生成对话数值。
 * 它是普通数量，不是事件状态枚举。
 */
MaryAnimalHealthyPregnancyDays GetAnimalHealthyPregnancyDays(
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index
);

/*
 * Returns the selected animal's age counter.
 * Parameters: animal_kind selects ANIMAL_KIND_*; animal_index follows
 * GetAnimalAffection's shared barn/separate coop roster rules, while horse and
 * dog ignore the index. The function returns the raw ten-bit counter
 * from record halfword 0x18 bits 0-9 (0-1023), or zero when resolution fails.
 * This is a numeric count, not a juvenile/adult selector; zero alone does not
 * prove that an animal exists.
 * Return value: the raw 0-1023 age counter, or zero when resolution fails.
 *
 * 返回指定动物的年龄计数。
 * 参数：animal_kind 选择 ANIMAL_KIND_*；animal_index 遵循 GetAnimalAffection
 * 的牛羊共享畜棚、鸡另用鸡舍名册规则，马和狗忽略该槽位参数。函数返回记录
 * 半字 0x18 的低十位原始计数
 * （0-1023），解析失败时返回零。这是普通数量，不是幼年／成年状态；零不能
 * 证明动物存在。
 * 返回值：0-1023 的原始年龄计数；解析失败时为零。
 */
MaryAnimalAgeDays GetAnimalAge(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Returns the selected animal's affection value.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is a zero-based
 * roster slot. Cows and sheep share the barn roster, not separate per-species
 * numbering; chickens use the coop roster. Horse and dog ignore animal_index.
 * An out-of-capacity slot, empty slot, or cow/sheep species mismatch resolves
 * to null. This is not a feeding-trough or pregnancy-stall selector.
 * Return value: the raw eight-bit affection value, or zero if animal resolution
 * fails. Normal additions limit it to 250, but this getter does not clamp
 * stored values 251-255. Zero alone does not prove that the animal exists.
 *
 * 返回指定动物的好感度。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为从 0 开始的动物名册槽位。
 * 牛羊共享畜棚名册，而非各物种分别编号；鸡使用鸡舍名册。马和狗忽略此参数。
 * 超出当前容量、空槽或牛羊种类不匹配时解析为空指针。此编号不是饲料槽编号，
 * 也不是南北怀孕隔间编号。
 * 返回值：存储的八位好感度值；无法解析到动物时返回零。正常增加操作将其
 * 限制到 250，但读取函数不会裁剪存档中的 251-255。零不能单独证明动物存在。
 */
MaryAnimalAffectionValue GetAnimalAffection(
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index
);

/*
 * Returns the selected animal's growth-stage value.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: the engine growth-stage value for horse, cow, sheep, chicken,
 * or dog as selected by animal_kind. Horse and dog ignore animal_index; cow,
 * sheep, and chicken use it to resolve a family-local record. An invalid kind,
 * a missing horse, or an unresolved livestock slot returns zero. When
 * animal_kind is a statically known symbol, the decompiler uses
 * MaryAnimalPetGrowthStage, MaryAnimalCowGrowthStage, MaryAnimalSheepGrowthStage, or
 * MaryAnimalChickenGrowthStage as appropriate.
 *
 * 返回指定动物的成长阶段值。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：由 animal_kind 选择的马、牛、羊、鸡或狗的引擎成长阶段值。马和狗
 * 忽略 animal_index；牛、羊、鸡用它解析各自类别内的记录。动物类别无效、马
 * 不存在或家畜槽无法解析时返回零。当 animal_kind 是静态可知的符号时，反编译器
 * 会按物种选用 MaryAnimalPetGrowthStage、MaryAnimalCowGrowthStage、MaryAnimalSheepGrowthStage
 * 或 MaryAnimalChickenGrowthStage。
 */
int GetAnimalGrowthStage(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected sheep has been sheared.
 * Parameter: sheep_index is the zero-based sheep slot in the shared barn.
 * Return value: nonzero when sheared; zero when not sheared or when the shared
 * barn slot does not resolve to a sheep record.
 *
 * 判断指定羊是否已经剪毛。
 * 参数：sheep_index 为共享牛羊棚中从 0 开始的羊槽位。
 * 返回值：已经剪毛时为非零；未剪毛或共享畜棚槽无法解析为羊记录时为零。
 */
MaryBool IsSheepSheared(MaryAnimalSlotIndex sheep_index);

/*
 * Counts cows, sheep, or chickens whose stored life state matches the requested
 * value.
 * Parameters: animal_kind is ANIMAL_KIND_COW, ANIMAL_KIND_SHEEP, or
 * ANIMAL_KIND_CHICKEN; life_state is LIVESTOCK_LIFE_STATE_*. Return value: the
 * number of non-null records within the current barn or coop capacity whose
 * stored state matches exactly. Horse, dog, and invalid kinds return zero. The
 * death-summary event uses the two death states before its cleanup removes
 * those records.
 *
 * 统计保存的生命状态等于指定值的牛、羊或鸡。
 * 参数：animal_kind 为 ANIMAL_KIND_COW、ANIMAL_KIND_SHEEP 或
 * ANIMAL_KIND_CHICKEN；life_state 为 LIVESTOCK_LIFE_STATE_*。
 * 返回值：在当前畜棚或鸡舍容量内，非空且保存状态完全匹配的记录数量。马、狗及
 * 无效类别返回零。动物死亡汇总事件会在清理相应记录前读取两种死亡状态。
 */
MaryAnimalCount CountAnimalsByLifeState(
    MaryAnimalKind animal_kind,
    MaryLivestockLifeState life_state);

/*
 * Builds the localized list of livestock that died from neglect and displays
 * it in the currently open talk window.
 * Parameters: none. Call before
 * RemoveLivestockDeadFromNeglect while life-state 2 records still exist.
 *
 * 生成因照料不当死亡的家畜本地化清单，并显示到当前已打开的对话框中。
 * 参数：无。应在生命状态 2 的记录仍存在时调用，并在之后调用
 * RemoveLivestockDeadFromNeglect。
 */
void ShowLivestockNeglectDeathSummary(void);

/*
 * Removes cow, sheep, and chicken records whose life state is
 * LIVESTOCK_LIFE_STATE_DIED_FROM_NEGLECT, including their live entity slots.
 * Parameters: none.
 *
 * 移除生命状态为 LIVESTOCK_LIFE_STATE_DIED_FROM_NEGLECT 的牛、羊、鸡记录，
 * 同时清理其场景实体槽位。
 * 参数：无。
 */
void RemoveLivestockDeadFromNeglect(void);

/*
 * Builds the localized list of livestock that died naturally and displays it
 * in the currently open talk window.
 * Parameters: none. Call before
 * RemoveNaturallyDeadLivestock while life-state 1 records still exist.
 *
 * 生成自然死亡家畜的本地化清单，并显示到当前已打开的对话框中。
 * 参数：无。应在生命状态 1 的记录仍存在时调用，并在之后调用
 * RemoveNaturallyDeadLivestock。
 */
void ShowNaturalLivestockDeathSummary(void);

/*
 * Removes cow, sheep, and chicken records whose life state is
 * LIVESTOCK_LIFE_STATE_DIED_NATURALLY, including their live entity slots.
 * Parameters: none.
 *
 * 移除生命状态为 LIVESTOCK_LIFE_STATE_DIED_NATURALLY 的牛、羊、鸡记录，
 * 同时清理其场景实体槽位。
 * 参数：无。
 */
void RemoveNaturallyDeadLivestock(void);

/*
 * Tests whether a barn slot contains a cow rather than another barn animal.
 * Parameter: barn_slot is the zero-based shared barn slot.
 * Return value: nonzero for a cow; zero for a sheep, an empty slot, or an
 * out-of-range slot.
 *
 * 判断共享牛羊棚槽位中是否为牛。
 * 参数：barn_slot 为从 0 开始的共享牛羊棚槽位。
 * 返回值：槽位中为牛时为非零；为羊、空槽或越界槽时均返回零。
 */
MaryBool IsCowAtBarnSlot(MaryAnimalSlotIndex barn_slot);

/*
 * Returns the number of cow records currently present within barn capacity.
 * Parameters: none.
 * Return value: cow count (0-8 before the barn upgrade, 0-16 afterwards).
 * All four native loops call the cow record resolver for each shared barn
 * slot and do not read the livestock life-state field; a pending-death record
 * remains counted until its cleanup removes the record.
 *
 * 返回当前畜棚容量范围内实际存在的牛记录数量。
 * 参数：无。
 * 返回值：牛数量（畜棚扩建前 0-8，扩建后 0-16）。四版原生循环都会逐一调用
 * 共享畜棚槽的牛记录解析器，不读取家畜生命状态；待清理的死亡记录在真正移除前
 * 仍会计数。
 */
MaryAnimalCount GetCowCount(void);

/*
 * Returns the number of sheep records currently present within barn capacity.
 * Parameters: none.
 * Return value: sheep count (0-8 before the barn upgrade, 0-16 afterwards).
 * All four native loops call the sheep record resolver for each shared barn
 * slot and do not read the livestock life-state field; a pending-death record
 * remains counted until cleanup.
 *
 * 返回当前畜棚容量范围内实际存在的羊记录数量。
 * 参数：无。
 * 返回值：羊数量（畜棚扩建前 0-8，扩建后 0-16）。四版原生循环都会逐一调用
 * 共享畜棚槽的羊记录解析器，不读取家畜生命状态；待清理的死亡记录在清理前
 * 仍会计数。
 */
MaryAnimalCount GetSheepCount(void);

/*
 * Returns the number of chicken records currently present within coop capacity.
 * Parameters: none.
 * Return value: chicken count (0-4 before the coop upgrade, 0-8 afterwards).
 * All four native loops call the chicken record resolver for each coop slot
 * and do not read the livestock life-state field; a pending-death record
 * remains counted until cleanup. Eggs in incubators are separate records and
 * are not included.
 *
 * 返回当前鸡舍容量范围内实际存在的鸡记录数量。
 * 参数：无。
 * 返回值：鸡数量（鸡舍扩建前 0-4，扩建后 0-8）。四版原生循环都会逐一调用
 * 鸡舍槽的鸡记录解析器，不读取家畜生命状态；待清理的死亡记录在清理前仍会
 * 计数。孵化箱中的蛋属于独立记录，不计入鸡数量。
 */
MaryAnimalCount GetChickenCount(void);

/*
 * Registers one owned animal as the entrant for its contest family.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the animal's
 * zero-based slot within that family. Cow and sheep share the barn selector;
 * chicken uses the coop selector; horse and dog ignore animal_index. The native
 * operation also removes/reinserts the live entity as needed and resets its
 * saved location to the appropriate home map. Invalid kinds do nothing.
 * Related calls: GetContestAnimalIndex reads the registered slot, and
 * ClearContestAnimal removes the registration after the contest or event.
 *
 * 将一只已拥有的动物登记为相应类别比赛的参赛动物。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该动物在类别内从 0 开始的槽位。
 * 牛羊共用畜棚选择器，鸡使用鸡舍选择器，马和狗忽略 animal_index。原生操作还会按
 * 需要移除并重新插入活动实体，并把保存位置重置到相应住所地图；无效类别不执行操作。
 * 联动调用：GetContestAnimalIndex 读取登记槽位；比赛或事件结束后由
 * ClearContestAnimal 清除登记。
 */
void SetContestAnimal(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Clears the registered contest entrant for one animal family.
 * Parameter: animal_kind is ANIMAL_KIND_*.
 * Side effects: clears the barn/coop selected-slot flag for livestock and
 * restores the selected animal entity to its home location; horse and dog are
 * likewise restored through their fixed entity IDs. Invalid kinds do nothing.
 * Related calls: SetContestAnimal registers the entrant and
 * GetContestAnimalIndex reads its slot.
 *
 * 清除一个动物类别中已经登记的比赛参赛动物。
 * 参数：animal_kind 为 ANIMAL_KIND_*。
 * 副作用：对家畜会清除畜棚／鸡舍的已选槽标志，并把所选动物实体恢复到住所位置；
 * 马和狗同样通过固定实体 ID 恢复。无效类别不执行操作。
 * 联动调用：SetContestAnimal 负责登记；GetContestAnimalIndex 读取其槽位。
 */
void ClearContestAnimal(MaryAnimalKind animal_kind);

/*
 * Returns the selected contest entrant slot for an animal family.
 * Parameter: animal_kind is ANIMAL_KIND_*.
 * Return value: the selected animal's zero-based family slot; horse and dog
 * return slot zero while their transferred entity is present. Returns -1 when
 * no entrant is registered/present or when animal_kind is invalid.
 *
 * 返回指定动物类别所选择的比赛参赛槽位。
 * 参数：animal_kind 为 ANIMAL_KIND_*。
 * 返回值：所选动物在该类别内从 0 开始的槽位；马和狗的已转移实体存在时返回槽位零。
 * 未登记／实体不存在或 animal_kind 无效时返回 -1。
 */
MaryAnimalSlotIndex GetContestAnimalIndex(MaryAnimalKind animal_kind);

/*
 * Writes the current catch's localized fish name into a numbered text-variable
 * slot.
 * Parameters: text_variable is zero-based, so 0 supplies {Var1}, 1
 * supplies {Var2}, and so on. Call after a catch has populated the current-fish
 * event state; GetCaughtFishSize and the two fish-property tests read that same
 * state.
 *
 * 将当前捕获鱼种的本地化名称写入编号文本变量槽。
 * 参数：text_variable 从 0 开始，因此 0 对应 {Var1}、1 对应 {Var2}，依此类推。
 * 应在捕获结果已经写入当前鱼事件状态后调用；GetCaughtFishSize 及两个鱼属性判断
 * 函数读取同一状态。
 */
void SetTextVariableToCaughtFishName(MaryTextVariableSlot text_variable);

/*
 * Returns the size of the fish currently being processed by the event.
 * Parameters: none.
 * Return value: current catch size in whole centimeters. All four vanilla
 * reward scripts divide the value by 100 into meters and the remaining
 * centimeters, and their US/JP text labels independently display m/cm.
 *
 * 返回当前事件正在处理的捕获鱼尺寸。
 * 参数：无。
 * 返回值：以整厘米为单位的当前捕获尺寸。四套原版奖励脚本都会除以 100，分别
 * 得到米数和剩余厘米数；US／JP 文本也独立显示 m／cm 单位。
 */
MaryFishSize GetCaughtFishSize(void);

/*
 * Tests whether the current catch is a maximum-size record.
 * Parameters: none.
 * Return value: nonzero for a maximum-size record; zero otherwise.
 *
 * 判断当前捕获是否为最大尺寸纪录。
 * 参数：无。
 * 返回值：属于最大尺寸纪录时为非零，否则为零。
 */
MaryBool IsCaughtFishMaximumSize(void);

/*
 * Tests whether the current catch belongs to the special Fish King group.
 * Parameters: none.
 * Return value: nonzero for a Fish King; zero otherwise. FoMT-US calls this
 * result a "River King" in its reward dialogue, while MFoMT-US says "king
 * fish"; the neutral callable name intentionally covers both official texts.
 *
 * 判断当前捕获是否属于特殊鱼王组。
 * 参数：无。
 * 返回值：属于鱼王时为非零，否则为零。FoMT-US 的奖励对话称其为 “River King”，
 * MFoMT-US 则写作 “king fish”；中性的 callable 名称有意兼容两套官方文本。
 */
MaryBool IsCaughtFishKing(void);

/*
 * Selects the partner index used by the upcoming Moon-Viewing Festival.
 * Parameters: none. Return value: FESTIVAL_MOON_VIEWING_PARTNER_*.
 *
 * A married player gets the spouse. Otherwise, each standard spouse candidate
 * is eligible while their rival-event count is at most 4. The selector starts
 * at 30000 love and only replaces the result when love is strictly greater,
 * so exactly 30000 does not qualify. Equal scores above that threshold use a
 * successive 50% replacement test; this is random but is not a uniform choice
 * among all tied candidates. If nobody qualifies, FoMT falls back to Popuri
 * and MFoMT falls back to Rick.
 *
 * 选择即将举行的赏月节所使用的同行对象索引。
 * 参数：无。返回值：FESTIVAL_MOON_VIEWING_PARTNER_*。
 *
 * 玩家已婚时返回配偶。未婚时，标准结婚候选人的情敌事件计数不超过 4 才能
 * 参选。选择器以 30000 爱情度为初值，只有严格大于当前值才会替换结果，因此
 * 恰好 30000 不满足条件。超过门槛且同分时逐次进行 50% 替换；这个过程具有
 * 随机性，但并不是在全部同分候选人之间作均匀随机选择。无人满足条件时，FoMT
 * 固定回退到 Popuri，MFoMT 固定回退到 Rick。
 */
MaryFestivalMoonViewingPartner SelectMoonViewingPartner(void);
/*
 * Rates the food currently held by the player for the active Cooking Festival
 * theme.
 * Parameters: none. Return value: FESTIVAL_COOKING_DISH_RATING_*.
 * Ratings excellent and great win in the vanilla result script; ineligible
 * means the held food is invalid or does not match the year's category.
 * For eligible food, score = stamina gain - fatigue gain, including the
 * food instance's signed bonuses. Each theme uses its own four thresholds;
 * a rating requires strictly exceeding its threshold, not merely equaling it.
 * Also updates VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID: empty/non-food hands
 * record ITEM_FOOD_NONE; a valid held food records its ID before category/rating
 * evaluation, even if that dish subsequently receives an ineligible rating.
 *
 * 按当前料理祭主题评价玩家手中持有的料理。
 * 参数：无。
 * 返回值：FESTIVAL_COOKING_DISH_RATING_*。
 * 原版结果脚本中“优秀”和“很棒”会获胜；“不合格”表示手持料理无效或不符合当年类别。
 * 对符合类别的食品，分数为体力回复量减去疲劳变化量，包含该食品实例的有符号
 * 附加修正值。每种主题有各自的四档阈值；必须严格大于阈值才能进入该档，
 * 恰好等于阈值时进入下一档。
 * 同时更新 VAR_COOKING_FESTIVAL_PLAYER_DISH_FOOD_ID：空手或非食品时记录
 * ITEM_FOOD_NONE；有效的手持食品在类别及评分判断前便记录其 ID，即使随后评分
 * 为不合格，也不代表该变量会恢复为 ITEM_FOOD_NONE。
 */
MaryFestivalCookingDishRating GetCookingFestivalDishRating(void);

/*
 * Selects the weighted mineral gift for Thomas's Winter 25 stocking event.
 * Parameters: none. Return value: MaryThomasStockingGift values 1-5; the
 * delivery script maps them to Mystrile, Orichalc, Moon Stone, Sand Rose, or
 * Alexandrite respectively.
 *
 * All four native handlers use the low eight bits of rand() >> 4 and the same
 * five-entry table. The exact outcome counts out of 256 are 126, 43, 43, 43,
 * and 1 in that order. The selector does not inspect the stocking, inventory,
 * delivery state, or existing gift; those checks belong to the event script.
 *
 * 为 Thomas 的冬 25 日袜子礼物事件按权重选择矿石礼物。
 * 参数：无。返回值：MaryThomasStockingGift 的 1-5；送礼脚本依次把它们映射为
 * 秘银、奥利哈钢、月亮石、沙漠玫瑰石和亚历山大石。
 *
 * 四版原生 handler 都取 rand() >> 4 的低八位，并使用相同的五项权重表。按
 * 256 种输入精确计算，各结果依次占 126、43、43、43、1 种。选择器本身不检查
 * 袜子、背包、送达状态或已有礼物；这些条件由事件脚本处理。
 */
MaryThomasStockingGift SelectThomasStockingGift(void);

/*
 * Draws one article ID from the weighted gift table used by post-marriage
 * spouse-present events.
 * Parameters: none. Return value: the selected
 * MaryItemArticleId, suitable for SetPlayerHeldArticle or its wrapped variant.
 * All four targets use the same table. Exact counts out of 256 are:
 * Amethyst 126, Topaz 43, Ruby 43, Diamond 43, and Pink Diamond 1. The
 * selector only draws and returns the article ID; it does not inspect or
 * modify inventory, wrapping, marriage state, or event state.
 *
 * 从婚后配偶赠礼事件使用的加权礼物表中抽取一个物品 ID。
 * 参数：无。
 * 返回值：选中的 MaryItemArticleId，可直接传给 SetPlayerHeldArticle
 * 或其包装物品版本。
 * 四个版本使用相同的表。按 256 种输入精确计算，紫水晶占 126，黄玉占 43，
 * 红宝石占 43，钻石占 43，粉红钻石占 1。选择器只抽取并返回物品 ID，不检查
 * 或修改背包、包装、婚姻状态或事件状态。
 */
MaryItemArticleId GetRandomSpouseGiftArticleId(void);

/*
 * Unlocks the next Van-shop music album made available by GameCube link
 * progress.
 * Parameters: none. Return value: 1 when the unlocked-album count
 * was increased, or 0 when all ten link albums were already unlocked. The
 * engine table maps these ten entries directly to ITEM_ARTICLE_ALBUM_1 through
 * ITEM_ARTICLE_ALBUM_10. The native state stores the unlocked prefix length at
 * offset 0x16 and the per-album removed/sold bitmap at offset 0x14. This
 * callable only increments the prefix length; it does not clear or restore
 * any album's bitmap state.
 *
 * 根据 GameCube 联动进度，解锁 Van 商店中下一张可出售的音乐唱片。
 * 参数：无。
 * 返回值：成功增加已解锁唱片数量时为 1；十张联动唱片均已解锁时为 0。
 * 引擎表会将这十项直接映射到 ITEM_ARTICLE_ALBUM_1 至 ITEM_ARTICLE_ALBUM_10。原生状态
 * 在偏移 0x16 保存已解锁前缀长度，在偏移 0x14 保存逐唱片移除／售出位图。
 * 本函数只增加前缀长度，不会清除或恢复任何唱片的位图状态。
 */
MaryBool UnlockNextVanAlbum(void);

/*
 * Tests whether all ten GameCube-link albums are currently available in
 * Van's shop.
 * Parameters: none. Return value: 1 only when every album slot
 * from Album 1 through Album 10 is unlocked and has not been removed from the
 * current shop inventory; otherwise 0. Concretely, the unlocked-prefix byte
 * must be at least 10 and all ten corresponding bits in the removed/sold
 * bitmap must be zero.
 *
 * 检查十张 GameCube 联动唱片当前是否全部可在 Van 商店购买。
 * 参数：无。
 * 返回值：仅当 Album 1 至 Album 10 的全部槽位均已解锁，且尚未从当前
 * 商店库存中移除时为 1；否则为 0。具体而言，已解锁前缀长度必须至少为 10，
 * 且移除／售出位图中对应十个位必须全部为 0。
 */
MaryBool AreAllVanAlbumsAvailable(void);

/*
 * Stores a preset nickname that the player's spouse will use for the player.
 * Parameter: nickname points to a script text entry containing the selected
 * name. For a custom typed nickname, use OpenNameEntry with
 * NAME_ENTRY_SPOUSE_NICKNAME instead.
 *
 * The destination is the player's second FixedStr<12>: at most 12 encoded
 * bytes are copied, followed by a zero terminator. If the source begins with
 * the two encoded bytes FF 21 (the current charmap's {Player} control token),
 * the handler ignores the rest of that source and copies the player's saved
 * name instead. The test is performed on encoded bytes, not on a hard-coded
 * source-language spelling.
 *
 * 保存配偶称呼玩家时所使用的预设昵称。
 * 参数：nickname 指向包含所选称呼的脚本文本。需要由玩家输入自定义昵称时，
 * 应改用 OpenNameEntry(NAME_ENTRY_SPOUSE_NICKNAME, ...) 。
 *
 * 目标字段是玩家结构中的第二个 FixedStr<12>：最多复制 12 个编码字节，随后补
 * 零终止字节。若来源开头两个编码字节为 FF 21（当前码表的 {Player} 控制符），
 * handler 会忽略来源的剩余内容，改为复制玩家已保存的本名。判断依据是编码字节，
 * 不是在程序中硬编码某种源文本写法。
 */
void SetPlayerNicknameForSpouse(const char *nickname);

/*
 * Moves the active field and player entity to the farmhouse bed position used
 * after overnight or event-ending transitions. This is not a position-only
 * helper: it first changes the active map to MAP_FARMHOUSE and installs the
 * packed destination, then positions ENTITY_PLAYER at the same coordinates.
 * Farmhouse upgrade levels 0, 1, and 2 select X coordinates 143, 263, and 327;
 * Y is always 112 and the player faces FACING_LEFT. The native table has only
 * these three entries, relying on the FarmHouse invariant that upgrade_level
 * stays in 0..2.
 * Parameters: none.
 *
 * 将当前场景及玩家实体移动到过夜或事件结束转场所使用的农舍床位。这不是只修改
 * 坐标的辅助函数：它先切换到 MAP_FARMHOUSE 并安装压缩目的地记录，再将
 * ENTITY_PLAYER 放到相同坐标。农舍升级等级 0、1、2 分别选择 X 坐标 143、263、
 * 327；Y 恒为 112，玩家朝向 FACING_LEFT。原生表只有这三项，依赖 FarmHouse 的
 * upgrade_level 始终处于 0..2 的内部不变量。
 * 参数：无。
 */
void PlacePlayerAtFarmhouseBed(void);

/*
 * Starts the fixed screen-color flash effect using GBA 5-bit RGB channels.
 * Parameters: red, green, and blue are each in the range 0..31. The engine
 * packs them as red | (green << 5) | (blue << 10) before updating the palette.
 * Fireworks events use the three full-intensity primary colors in sequence.
 * These are channel intensities, not a duration or speed. The native packing
 * performs shifts and ORs without clamping or masking each channel; 0..31 is
 * the intended input domain, not an enforced runtime bound. Out-of-range
 * inputs can affect adjacent color bits and must not be silently normalized.
 * All four targets reset the effect counter and use the same 120-entry
 * blend-strength sequence, then clear the effect on the next update. This
 * is 120 effect updates, not a caller-specified duration; calling again
 * reinitializes the color and counter rather than queuing another flash.
 *
 * 使用 GBA 的 5 位 RGB 通道启动固定的屏幕闪色效果。
 * 参数：red、green、blue 的范围均为 0..31。引擎按
 * red | (green << 5) | (blue << 10) 打包后更新调色板；烟火事件会依次使用
 * 三种满强度原色。
 * 三个参数都是颜色强度，不是持续时间或速度。原生打包直接移位并按位或，
 * 不会逐通道截断或限制范围；0..31 是预期输入域，并非运行时强制边界。
 * 越界输入可能影响相邻颜色位，编译器不得静默归一化这些原始值。
 * 四个版本均重置效果计数，使用相同的 120 项混合强度序列，随后在下一次
 * 更新时清理效果。这是 120 次效果更新，而非调用者指定的持续时间；再次
 * 调用会重新初始化颜色与计数，不会把另一次闪色追加到队列中。
 */
void FlashScreenColor(MaryRgb5Channel red, MaryRgb5Channel green, MaryRgb5Channel blue);

/*
 * Rebuilds the current map's entity manager at the start of the new-day
 * script. This reloads map actors for the new calendar state and applies a
 * preserved overnight return position when one is active.
 * Parameters: none.
 * All four native paths first run the game-state daily update, including
 * farm/NPC updates and visit-counter guard resets and affection rewards.
 * It is not a side-effect-free actor reload; do not call it merely to refresh
 * an actor's graphics. Reset/reward processing is tied to this update call,
 * not an autonomous timer attached to each visit counter.
 *
 * 在新一天脚本开始时重建当前地图的实体管理器。它会按照新的日期状态重新载入
 * 地图角色，并在存在已保存的过夜返回位置时恢复该位置。
 * 参数：无。
 * 四个版本的原生路径都会先执行游戏状态的每日更新，包括农场／NPC 更新、访问
 * 计数重复标记重置和爱情度奖励结算。它不是无副作用的角色重新载入操作。
 * 不应仅为刷新角色图像而调用。重置／奖励处理由这条更新调用链触发，
 * 不是访问计数器各自附带的独立定时器。
 */
void RebuildMapEntitiesForNewDay(void);

/*
 * Cures every currently sick livestock entity in the farm-animal entity range.
 * Parameters: none. The native routine scans entity IDs 0x2E through 0x45,
 * skips missing entities, tests each resolved livestock object's sick flag,
 * and calls ResetSick only when set. ResetSick clears both the sick flag and
 * accumulated sick-day counter. It does not clear unhappiness or change
 * affection, feeding, pregnancy, age, or festival-winner state. The
 * shooting-star event uses this for the healthy-animals wish; healthy
 * livestock is left unchanged.
 *
 * 治愈农场动物实体范围内当前所有生病的家畜。
 * 参数：无。原生例程扫描实体 ID 0x2E 到 0x45，跳过不存在的实体，检查每个
 * 成功解析的家畜对象的生病标志，并且只在该标志置位时调用 ResetSick。
 * ResetSick 会同时清除生病标志与累计生病天数；不会清除“不高兴”，也不会改变
 * 好感度、喂食、怀孕、年龄或祭典优胜状态。射星事件的“动物健康”愿望调用
 * 此函数；健康家畜不会发生变化。
 */
void CureAllSickLivestock(void);

/*
 * Adds the same signed affection amount to every owned farm animal: dog,
 * horse, chickens, cows, and sheep.
 * Parameter: amount is the signed affection
 * delta. The dog is always updated; the horse is updated only when present;
 * chicken and shared cow/sheep rosters are traversed up to their currently
 * unlocked capacities, skipping empty slots. Sick, unhappy, pregnant, young,
 * or festival-winning animals are not filtered out. Every resolved animal
 * uses the same signed-add operation as AddAnimalAffection and clamps its final
 * affection to 0-250. Church-confession outcomes currently call this with a
 * positive value.
 *
 * 为所有已拥有的农场动物增加相同的有符号好感变化量，包括狗、马、鸡、牛和羊。
 * 参数：amount 为有符号好感变化量。狗始终更新；马仅在存在时更新；鸡舍名册及
 * 牛羊共用名册会遍历到当前已解锁容量并跳过空槽。函数不会排除生病、不高兴、
 * 怀孕、幼年或祭典优胜动物。每个成功解析的动物都使用与 AddAnimalAffection
 * 相同的有符号加法，并把最终好感度限制到 0-250。当前教堂忏悔结果会传入正值。
 */
void AddAffectionToAllFarmAnimals(MaryAnimalAffectionDelta amount);

/*
 * Enables the one-shot double payout awarded by the shooting-star event's
 * shipping wish.
 * Parameters: none. This function idempotently sets a dedicated byte to 1;
 * repeated calls do not stack. At the next shipping settlement, the engine
 * adds that settlement's accumulated shipping value to the player's money a
 * second time, clears the byte, and resets the accumulated shipping value.
 * It does not immediately ship an item or permanently change product prices.
 *
 * 启用射星事件“提高出货收入”愿望所奖励的一次性双倍结算。
 * 参数：无。该函数会幂等地把专用字节置为 1，重复调用不会叠加。下一次出货结算
 * 时，引擎会把本次累计出货额再次加入玩家金钱，随后清除此字节并清空累计出货额。
 * 它不会立即出货物品，也不会永久修改产品价格。
 */
void EnableShootingStarShippingBonus(void);

/*
 * Returns the cumulative shipped count for a product.
 * Parameter: product_id is the target-specific MaryProductId. This is the
 * complete ShippingBin table domain 0x00..0x66, not MaryItemFoodId or
 * MaryItemArticleId. Values above 0x66 return zero. The native comparison is
 * signed and does not reject negative raw integers, which can index before
 * the table; callers must therefore use MaryProductId rather than treating
 * this as a generally range-safe integer API.
 * Return value: the entry's 31-bit cumulative shipped amount when its display
 * flag is enabled, otherwise zero. Shipping saturates the stored count at
 * 1,000,000,000.
 *
 * 返回指定产品的累计出货量。
 * 参数：product_id 为目标版本完整的 ShippingBin 表编号域 0x00..0x66；它不是
 * MaryItemFoodId 或 MaryItemArticleId。大于 0x66 的值返回零。原生比较使用有符号条件，
 * 不会拒绝负的原始整数，负值可能索引到表之前；调用方必须使用 MaryProductId，
 * 不能把它当作对任意整数都安全的接口。
 * 返回值：显示标志已启用时返回该表项的 31 位累计出货量，否则返回零。出货时
 * 存储计数会在 1,000,000,000 饱和。
 */
MaryProductShippedCount GetAmountShipped(MaryProductId product_id);

/*
 * Initializes the player's child record when needed, then creates and enables
 * the child actor entity (entity ID 35) for the current scene.
 * Parameters:
 * none. Childbirth scripts call this immediately before positioning the baby;
 * existing child data is preserved rather than initialized again.
 * An existing scene entity in slot 35 is destroyed and replaced; preserving
 * the child record does not mean preserving the existing runtime entity.
 * Requires the ScriptEngine scene-context pointer to be present; if it is
 * null, the native handler returns without initializing the child record.
 *
 * 在需要时初始化玩家孩子的资料，随后为当前场景创建并启用孩子角色实体
 *（实体 ID 35）。
 * 参数：无。生育事件会在设置婴儿位置前调用此函数；已经存在
 * 的孩子资料不会被重复初始化。
 * 场景中槽 35 的旧实体会被销毁并替换；保留孩子资料不等于保留原运行时实体。
 * 要求 ScriptEngine 的场景上下文指针存在；为空时，原生 handler 直接返回，
 * 不会初始化孩子记录。
 */
void CreatePlayerChildEntity(void);

/*
 * Sets a runtime entity's explicitly selected map and target coordinates.
 * Parameters: entity_id is the runtime actor ID; map_id is a
 * MaryMapId; x and y are target map coordinates. Unlike SetEntityPosition,
 * this uses map_id rather than the current scene map. The dispatcher passes
 * zero as the actor-facing argument. This is not a guarantee of save-file
 * persistence or entity creation. For ENTITY_PLAYER it reaches the same
 * family-specific placement side-effect path as SetEntityPosition: mine maps
 * may update the shared deepest-floor record, FoMT's library second floor may
 * advance Mary's guarded visit counter, and MFoMT's church may advance
 * Cliff's guarded visit counter subject to the romance-event and Music
 * Festival exclusions documented on SetEntityPosition.
 *
 * 设置运行时实体的指定地图和目标坐标。
 * 参数：entity_id 为运行时
 * 角色 ID；map_id 为 MaryMapId；x、y 为目标地图坐标。它与
 * SetEntityPosition 不同，使用 map_id 而非当前场景地图。分派器把角色朝向参数
 * 设为零。本调用不保证写入存档或创建实体。对 ENTITY_PLAYER 而言，它会进入与
 * SetEntityPosition 相同的版本专用定位副作用路径：矿地图可能更新两座矿共用的
 * 历史最深层记录；FoMT 图书馆二楼可能推进玛丽的受保护访问计数；MFoMT 教堂则
 * 可能在满足 SetEntityPosition 所述恋爱事件及音乐节排除条件后推进克里夫的
 * 受保护访问计数。
 */
void RelocateEntityToMap(
    MaryEntityId entity_id,
    MaryMapId map_id,
    MaryMapSpaceX x,
    MaryMapSpaceY y);

/*
 * Creates or replaces the first-sunrise child effect owned by the scene's
 * fixed event-effect host in entity slot 0x5D. It does not create that scene
 * entity. The host must already exist; the native routine does not null-check
 * the entity returned by the scene lookup.
 * Parameters: none. Call PlayNewYearSunriseEffect next and release the child
 * effect with DestroyNewYearSunriseEffect afterward.
 *
 * 在场景固定事件特效宿主的实体槽 0x5D 内创建或替换“新年首次日出”子特效；
 * 它不会创建该场景实体。宿主必须已经存在，原生函数不会检查场景查找返回的
 * 实体是否为空。
 * 参数：无。随后应调用 PlayNewYearSunriseEffect，结束后再用
 * DestroyNewYearSunriseEffect 释放子特效。
 */
void CreateNewYearSunriseEffect(void);

/*
 * Starts the previously created first-sunrise child animation and switches
 * script execution to the native sunrise wait state 0x1C until the child
 * reports completion. The same fixed host entity 0x5D must exist.
 * Parameters: none.
 *
 * 启动此前创建的新年首次日出子动画，并把脚本执行切换到原生日出等待状态
 * 0x1C，直至子对象报告完成。固定宿主实体 0x5D 同样必须存在。
 * 参数：无。
 */
void PlayNewYearSunriseEffect(void);

/*
 * Destroys and clears the first-sunrise child effect after the New Year
 * transition. This does not destroy the fixed scene host entity 0x5D.
 * Parameters: none.
 *
 * 在跨年转场结束后销毁并清空首次日出子特效；不会销毁固定场景宿主实体 0x5D。
 * 参数：无。
 */
void DestroyNewYearSunriseEffect(void);

/*
 * Replaces the reusable star-sparkle particle collection owned by fixed host
 * entity 0x5D and switches script execution to native wait state 0x1B.
 * Scripts use it both for the shooting-star night and for the Gourmet's
 * highest cooking-festival reaction. It is independent of the sunrise child
 * slot, but requires the same host entity to exist.
 * Parameters: none.
 *
 * 替换固定宿主实体 0x5D 所持有的可复用星光粒子集合，并把脚本执行切换到原生
 * 等待状态 0x1B。脚本会在流星雨之夜及美食家对料理祭最高评价时使用它。该集合
 * 与日出子对象使用不同槽位，但同样要求宿主实体已存在。
 * 参数：无。
 */
void PlayStarSparkleEffect(void);

/*
 * Counts recipe records whose learned flag is set. The cooking menu uses a
 * nonzero result to decide whether the recipe-list screen can be opened.
 * Each marked recipe record contributes one, regardless of how many times
 * it was cooked. The six separately stored failed-dish flags are not counted.
 * This is not the pending Lou/Ruby reward-dialogue counter. MFoMT also uses
 * this native record count when updating its all-recipes completion latch.
 * Parameters: none.
 * Return value: the number of known recipes.
 *
 * 统计已设置“学会”标志的料理记录。料理菜单以结果是否非零决定能否打开菜谱
 * 列表画面。
 * 每条已标记的菜谱只计一次，不按烹饪次数累计；另外保存的六种失败料理标记
 * 不计入此数量。它也不是 Lou／Ruby 待处理奖励对话计数。女孩版还会使用同一
 * 原生记录计数来更新全部料理完成标记。
 * 参数：无。
 * 返回值：已经掌握的菜谱数量。
 */
MaryKnownRecipeCount GetKnownRecipeCount(void);

/*
 * Generates the procedural 28-by-28 layout for one mine floor. Before calling
 * the native generator, it looks up the basket, farm dog, and runtime entity
 * 75. If an actor is currently on the requested mine floor, the actor's four
 * tile-corner coordinates are passed as occupied cells (at most 12 cells in
 * total). These are mobile/runtime actor exclusions, not fixed entrance or
 * exit markers.
 * Parameters:
 * mine_kind selects MINE_SPRING or MINE_LAKE; floor_index is zero-based within
 * that mine's 256-floor range. The wrapper assumes the enum and floor-index
 * domains are valid; out-of-domain raw integers are still forwarded to the
 * native generator rather than rejected locally.
 *
 * 为指定矿层生成程序化的 28×28 布局。调用原生生成器前，会依次查找篮子、
 * 农场狗和运行时实体 75；若该实体当前位于目标矿层，则把实体矩形的四个格子
 * 角坐标作为占用位置传入，合计最多 12 格。这些是可移动／运行时实体的排除
 * 坐标，并非固定入口或出口标记。
 * 参数：mine_kind 选择 MINE_SPRING 或 MINE_LAKE；floor_index 是该矿场
 * 256 层范围内从 0 开始的层号。包装层假定枚举值和层号有效；超出范围的原始
 * 整数不会在本层被拒绝，而会继续传给原生生成器。
 */
void GenerateMineFloorLayout(MaryMineKind mine_kind, MaryMineFloorIndex floor_index);

/*
 * Builds and enters the next deeper floor of the current spring or lake mine.
 * It derives the target map and zero-based floor index from the current mine
 * map, gathers the basket, farm dog, and runtime entity 75 footprints already
 * assigned to that target floor, generates the new floor without covering
 * those cells, finds the generated entrance tile, and starts the map change
 * with the player positioned at that entrance.
 * Parameters: none.
 * The calling script handles the fade and the one-minute time advance around
 * this operation. The routine is a no-op outside the two mine-floor map ranges
 * and has no local deepest-floor guard; vanilla floor data prevents descent
 * from being requested beyond the supported mine depth.
 *
 * 构建并进入当前泉矿或湖矿的下一层。它根据当前矿层地图求出目标地图和从零
 * 开始的层号，收集已经分配到目标层的篮子、农场狗与运行时实体 75 的占用
 * 坐标，在不覆盖这些格子的前提下生成新楼层，查找生成的入口格，并把玩家
 * 定位到入口后开始地图切换。
 * 参数：无。调用脚本负责在该操作前后处理淡入淡出以及经过一分钟的时间更新。
 * 当前地图不属于两段矿层范围时，本函数不执行操作；函数内部没有最深层保护，
 * 原版依靠楼层数据避免在支持深度以外请求继续下行。
 */
void DescendMineFloor(void);

/*
 * Returns whether the selected cursed tool's curse is still active. The
 * engine maps the six cursed tool IDs to six dedicated curse-state records
 * in sickle, hoe, axe, hammer, watering-can, and fishing-rod order.
 * Parameter: tool_id is one of the six cursed ITEM_TOOL_* IDs. Return value:
 * nonzero while that tool remains cursed; zero otherwise.
 * Passing any other raw integer is unsafe: the native mapper falls back to
 * the cursed-hoe record instead of reporting an invalid ID.
 *
 * 返回所选诅咒农具的诅咒是否仍然生效。引擎会把六种诅咒农具 ID 映射到六个
 * 独立的诅咒状态记录，顺序为镰刀、锄头、斧头、锤子、洒水壶、钓竿。
 * 参数：tool_id 为六种诅咒 ITEM_TOOL_* ID 之一。
 * 返回值：仍受诅咒时为非零，否则为零。
 * 传入其他原始整数并不安全：原生映射器会回落到诅咒锄头记录，而不是报告
 * 无效 ID。
 */
MaryBool IsToolCursed(MaryItemToolId tool_id);

/*
 * Advances the consecutive-equipped-day lift condition for the cursed sickle
 * or cursed hammer. Each call increments that tool's byte counter; on the
 * tenth call it clears the active-curse byte, marks the tool blessed, and
 * returns TRUE. All other cursed tools return FALSE without advancing their
 * own lift condition. The sleeping script calls this once after a completed
 * night, and uses TRUE to present the curse-lift sequence.
 * Parameter: tool_id must be ITEM_TOOL_SICKLE_CURSED or ITEM_TOOL_HAMMER_CURSED.
 * Return value: TRUE exactly when this call completes the tenth-day lift;
 * FALSE otherwise.
 *
 * 推进诅咒镰刀或诅咒锤子的“连续装备天数”解除条件。每次调用把对应字节计数
 * 加一；第十次会清除诅咒生效字节、标记工具已祝福并返回 TRUE。其他诅咒农具
 * 返回 FALSE，且不会推进各自的解除条件。睡眠脚本在完成一夜后调用一次，并以
 * TRUE 结果播放解除诅咒演出。
 * 参数：tool_id 必须为 ITEM_TOOL_SICKLE_CURSED 或 ITEM_TOOL_HAMMER_CURSED。
 * 返回值：本次调用恰好完成第十天解除条件时为 TRUE，否则为 FALSE。
 */
MaryBool AdvanceCursedToolLiftProgress(MaryItemToolId tool_id);

/*
 * Applies one paid church/confessional removal visit to the cursed hoe or
 * cursed watering can. Their counters complete on the tenth call, which
 * clears the active curse, marks the tool blessed, and returns TRUE. A cursed
 * sickle or hammer instead has its consecutive-equipped-day counter reset to
 * zero; axe and fishing rod are unchanged. Other paths return FALSE. The
 * confessional script calls this only after accepting and charging the fee.
 * Parameter: tool_id is one of the six cursed ITEM_TOOL_* IDs.
 * Return value: TRUE exactly when this visit completes the hoe or watering-can
 * lift; FALSE for the reset, no-change, and incomplete paths.
 *
 * 对诅咒锄头或诅咒洒水壶应用一次付费的教堂／忏悔室解除次数；第十次调用会
 * 清除诅咒生效字节、标记工具已祝福并返回 TRUE。若传入诅咒镰刀或诅咒锤子，
 * 则把其连续装备天数计数清零；斧头和钓竿不变，其余路径返回 FALSE。忏悔室
 * 脚本只在确认并扣除费用后调用本函数。
 * 参数：tool_id 为六种诅咒 ITEM_TOOL_* ID 之一。
 * 返回值：本次访问恰好完成锄头或洒水壶解除时为 TRUE；重置、无变化及尚未完成
 * 路径均为 FALSE。
 */
MaryBool AttemptChurchCursedToolRemoval(MaryItemToolId tool_id);

/*
 * Creates a temporary event icon in an event-local scene slot. Coordinates
 * use the current map's local pixel space, like entity placement and camera
 * movement; layer selects the two-bit OBJ display priority (0 is highest,
 * 3 lowest relative to backgrounds; it is not a map-layer ID), and icon_id may
 * be supplied by GetFoodIconId() or as an original numeric engine icon ID.
 * Scripts may keep several slots alive simultaneously, so slot is an
 * event-local handle rather than an icon ID. This coordinate interpretation
 * is independently supported by all four script sets: family scenes place
 * actors near (292, 84) and (327, 114), then create table-food icons at
 * (288, 122/123) and (304, 124). X=288 and X=304 also exceed the GBA's
 * 240-pixel display width, ruling out fixed visible-screen coordinates.
 * Parameters: slot selects EVENT_ICON_SLOT_*; x and y are current-map local
 * coordinates; layer is EVENT_ICON_LAYER_*; icon_id is an engine icon ID.
 *
 * 在带编号的场景槽中创建临时事件图标。x、y 与实体定位及镜头移动一样，使用
 * 当前地图的局部像素坐标；layer 选择两位 OBJ 显示优先级（相对背景 0 最高、
 * 3 最低，不是地图层 ID）；icon_id 可由 GetFoodIconId()
 * 提供，也可使用原始引擎图标数字 ID。
 * 脚本可以同时保留多个槽，因此 slot 是事件局部句柄，而不是图标 ID。
 * 四套脚本提供了相互独立的坐标证据：家庭场景先把人物放在 (292, 84) 与
 * (327, 114) 附近，再把餐桌食物图标创建于 (288, 122/123) 和 (304, 124)。
 * X=288、304 也超过 GBA 画面宽度 240，因此不可能是固定可见屏幕坐标。
 * 参数：slot 选择 EVENT_ICON_SLOT_*；x、y 是当前地图局部坐标；layer 为
 * EVENT_ICON_LAYER_*；icon_id 为引擎图标 ID。
 */
void CreateEventIcon(
    MaryEventIconSlot slot,
    MaryMapSpaceX x,
    MaryMapSpaceY y,
    MaryEventIconLayer layer,
    MaryEventIconId icon_id);

/*
 * Removes the temporary event icon stored in an event-local scene slot.
 * Parameter: slot is the same handle passed to CreateEventIcon(). Removing one
 * slot does not affect icons kept in other slots.
 *
 * 删除保存在事件局部场景槽中的临时图标。
 * 参数：slot 为传给 CreateEventIcon() 的同一句柄；删除一个槽不会影响其他槽中
 * 保留的图标。
 */
void RemoveEventIcon(MaryEventIconSlot slot);

/*
 * Maps a food ID to the icon used by menus and message presentation.
 * Parameter: food_id is ITEM_FOOD_* or the exact original food ID.
 * Return value: the engine icon ID associated with that food. Valid IDs are
 * 0x00-0xAA on all four targets. An invalid ID returns icon 428 in FoMT and
 * icon 439 in MFoMT; it does not report an error or return zero.
 *
 * 将食品 ID 映射为菜单与消息显示使用的图标。
 * 参数：food_id 为 ITEM_FOOD_* 或精确原始食品 ID。
 * 返回值：该食品关联的引擎图标 ID。四个目标的有效 ID 均为 0x00～0xAA。
 * 无效 ID 在 FoMT 返回图标 428，在 MFoMT 返回图标 439；不会报错或返回零。
 */
MaryEventIconId GetFoodIconId(MaryItemFoodId food_id);

/*
 * Maps an article ID to the icon used by menus and event presentation.
 * Parameter: article_id is ITEM_ARTICLE_* or the exact original article ID.
 * Return value: the engine icon ID associated with that article. FoMT accepts
 * 0x00-0x5E and falls back to icon 457; MFoMT accepts 0x00-0x69 and falls
 * back to icon 468. Invalid IDs do not report an error or return zero.
 *
 * 将物品 ID 映射为菜单与事件显示使用的图标。
 * 参数：article_id 为 ITEM_ARTICLE_* 或精确原始物品 ID。
 * 返回值：该物品关联的引擎图标 ID。FoMT 接受 0x00～0x5E，无效时返回
 * 图标 457；MFoMT 接受 0x00～0x69，无效时返回图标 468。无效 ID 不会
 * 报错或返回零。
 */
MaryEventIconId GetArticleIconId(MaryItemArticleId article_id);

/*
 * Maps a tool ID to the icon used by menus and event presentation.
 * Parameter: tool_id is ITEM_TOOL_* or the exact original tool ID.
 * Return value: the engine icon ID associated with that tool. Valid IDs are
 * 0x00-0x50 on all four targets. An invalid ID returns icon 457 in FoMT and
 * icon 468 in MFoMT; it does not report an error or return zero.
 *
 * 将工具 ID 映射为菜单与事件显示使用的图标。
 * 参数：tool_id 为 ITEM_TOOL_* 或精确原始工具 ID。
 * 返回值：该工具关联的引擎图标 ID。四个目标的有效 ID 均为 0x00～0x50。
 * 无效 ID 在 FoMT 返回图标 457，在 MFoMT 返回图标 468；不会报错或返回零。
 */
MaryEventIconId GetToolIconId(MaryItemToolId tool_id);

/*
 * Verified retail no-op retained by the farming-tutorial bytecode. The VM
 * pushes five values that resemble grid x/y, tile state, object ID, and
 * variant, but all four ROMs dispatch this slot directly to the common return
 * block. The handler does not pop or inspect them and changes no field state.
 * Parameters: unused_operand_1 through unused_operand_5 preserve the five
 * original stack operands; none is consumed by the native handler. Their
 * positional values often resemble tutorial grid fields, but the no-op cannot
 * establish those as native parameter types.
 *
 * 种田教程字节码中保留的已确认零售版空操作。VM 会压入五个看似网格 x/y、
 * 格子状态、对象 ID 与变体的值，但四个 ROM 都把该槽直接派发到共用返回块；
 * handler 不弹出也不检查这些值，不会改变农田状态。
 * 参数：unused_operand_1 至 unused_operand_5 保留五个原始栈操作数；原生
 * handler 不消费其中任何一个。各位置数值经常看似教程网格字段，但空操作本身
 * 无法证明它们是原生参数类型。
 */
void NoOpTutorialFieldTile(
    int unused_operand_1,
    int unused_operand_2,
    int unused_operand_3,
    int unused_operand_4,
    int unused_operand_5
);

/*
 * Verified three-operand retail no-op adjacent to NoOpTutorialFieldTile. Its
 * coordinate-like operands are retained only to preserve the original stack
 * program and emitted bytes.
 * Parameters: unused_operand_1 through unused_operand_3 preserve the three original stack
 * operands; none is consumed by the native handler.
 *
 * 与 NoOpTutorialFieldTile 相邻的已确认三操作数零售版空操作。看似坐标与状态的
 * 操作数仅为保存原始栈程序及编译字节而保留。
 * 参数：unused_operand_1 至 unused_operand_3 保留三个原始栈操作数；原生
 * handler 不消费它们。
 */
void NoOpTutorialFieldObject(int unused_operand_1, int unused_operand_2, int unused_operand_3);

/*
 * Verified three-operand retail no-op found in the chicken tutorial. The
 * coordinate- and slot-like values are not consumed by the native handler.
 * Parameters: unused_operand_1 through unused_operand_3 preserve the three original stack operands.
 *
 * 养鸡教程中的已确认三操作数零售版空操作。看似坐标与槽位的值不会被原生
 * handler 消费。
 * 参数：unused_operand_1 至 unused_operand_3 保留三个原始栈操作数。
 */
void NoOpTutorialEggDefinition(int unused_operand_1, int unused_operand_2, int unused_operand_3);

/* Verified one-operand retail no-op paired with NoOpTutorialEggDefinition.
 * Parameter: unused_operand preserves the original stack operand; the native
 * handler does not consume it.
 *
 * 与 NoOpTutorialEggDefinition 配对的已确认单操作数零售版空操作。
 * 参数：unused_operand 保留原始栈操作数；原生 handler 不消费它。
 */
void NoOpTutorialEggSelection(int unused_operand);

/*
 * Puts the player into the scripted state for holding an actor graphic above
 * their head and waits for the transition.
 * Parameter: animation_id selects a target-specific MaryAnimationId graphic
 * animation by its single canonical constant name. Chicken tutorials call
 * this after removing the live chicken entity;
 * FoMT and MFoMT use different physical IDs for ANIMATION_CHICKEN_HELD.
 *
 * 让玩家进入把角色图形举在头顶的脚本状态，并等待状态转场完成。
 * 参数：animation_id 通过唯一规范常量名选择目标版本的 MaryAnimationId 图形动画。
 * 养鸡教程会在删除活动鸡实体后调用本函数；FoMT 与 MFoMT 的
 * ANIMATION_CHICKEN_HELD 使用不同物理编号。
 */
void BeginHoldingActorGraphic(MaryAnimationId animation_id);

/*
 * Verified three-operand retail no-op found before animal animation setup in
 * tutorial scripts. All four retail script sets consistently pass tutorial
 * entity slots 95-97 as the first operand, so that operand retains MaryEntityId.
 * The remaining growth- and actor-kind-like values are not consumed and do not
 * have independently verified native domains, so they deliberately remain int.
 * Actual entity setup is performed by other engine paths.
 * Parameters: entity_id preserves the verified tutorial entity; growth_stage
 * and animal_actor_kind preserve raw stack operands. The native handler
 * consumes none of the three values.
 *
 * 教程脚本在设置动物动画前保留的已确认三操作数零售版空操作。四套零售脚本都
 * 一致地把教程实体槽 95-97 作为第一个操作数，因此该操作数保留 MaryEntityId。
 * 其余看似成长阶段和角色类别的值既不被消费，也没有独立确认的原生取值域，故意
 * 继续使用 int；实际实体初始化由其他引擎路径完成。
 * 参数：entity_id 保留已确认的教程实体；growth_stage 与 animal_actor_kind 仅保留
 * 原始栈操作数，原生 handler 不消费三者。
 */
void NoOpAnimalEventEntityInitialization(
    MaryEntityId entity_id,
    int growth_stage,
    int animal_actor_kind
);

/*
 * Writes the localized name of product_id into the requested text-variable
 * slot. The product ID is normally supplied by GetEventContextValue() in the
 * Harvest Goddess cumulative-shipment events.
 * Parameters: text_variable is TEXT_VARIABLE_1 through TEXT_VARIABLE_4;
 * product_id is PRODUCT_* in the complete shipping-product domain.
 *
 * 将 product_id 对应的本地化产品名称写入指定文本变量槽。在女神累计出货事件中，
 * 产品 ID 通常由 GetEventContextValue() 提供。
 * 参数：text_variable 为 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4；product_id 为
 * 完整出货产品域中的 PRODUCT_*。
 */
void SetTextVariableToProductName(
    MaryTextVariableSlot text_variable,
    MaryProductId product_id
);

/*
 * Returns the integer context value attached by the event dispatcher to the
 * currently running script. Its meaning is event-specific: confirmed uses
 * include the current day's shipping-bin value in Zack's collection event and
 * a product ID in the Harvest Goddess cumulative-shipment events. Do not treat
 * this as a globally fixed shipping-value or product-ID getter.
 * Parameters: none.
 * Return value: the raw event-specific integer context.
 *
 * 返回事件派发器附加到当前运行脚本的整数上下文值，其含义由事件决定。已确认的
 * 用法包括：Zack 收货事件中的当天出货箱总值，以及女神累计出货事件中的产品 ID。
 * 不应将其理解为全局固定的“读取出货额”或“读取产品 ID”函数。
 * 参数：无。
 * 返回值：当前事件专用的原始整数上下文。
 */
int GetEventContextValue(void);

/*
 * Cycles the equipped-tool selection backward through the tool rucksack. If
 * the newly selected tool is one of the six cursed tools, cycling continues;
 * the operation stops at the first non-cursed tool or after ten cycles.
 * Parameters: none. The church uses this after a paid curse-removal attempt,
 * so the player is not left holding a cursed tool when another selectable
 * tool is available. Despite the historical symbol name, it does not seek or
 * select another cursed tool.
 *
 * 在工具背包中向后轮换当前装备选择。若新选中的仍是六种诅咒农具之一，就
 * 继续轮换；遇到第一把非诅咒工具或完成十次轮换时停止。
 * 参数：无。教堂在一次付费解除尝试后调用它，使存在其他可选工具时玩家不会
 * 继续拿着诅咒工具。与历史符号名的含义相反，它不会寻找或选择另一把诅咒
 * 工具。
 */
void CycleBackwardToNonCursedTool(void);
