/* Shared ordered callable IDs. Select one MARY_* target in the .mary.c source.
 *
 * Evidence provenance: the current `fomt` source tree represents FoMT-US.
 * MFoMT-US and MFoMT-JP callable semantics are verified separately from
 * their ROM handlers and scripts.
 *
 * 共享的有序 callable ID。请在 .mary.c 源文件中选择一个 MARY_* 目标。
 *
 * 证据来源说明：当前 `fomt` 源码树代表 FoMT-US；MFoMT-US 与 MFoMT-JP callable
 * 的语义必须分别通过各自 ROM 原生处理函数和脚本独立验证。 */
#if !(defined(MARY_FOMT_US) || defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))
#error Select exactly one MARY_* target
#endif
#if (defined(MARY_FOMT_US) && (defined(MARY_MFOMT_US) || defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_MFOMT_US) && (defined(MARY_FOMT_JP) || defined(MARY_MFOMT_JP))) || (defined(MARY_FOMT_JP) && defined(MARY_MFOMT_JP))
#error Select exactly one MARY_* target
#endif
#if defined(MARY_FOMT_US)
#define MARY_FOMT
#define MARY_US
#elif defined(MARY_MFOMT_US)
#define MARY_MFOMT
#define MARY_US
#elif defined(MARY_FOMT_JP)
#define MARY_FOMT
#define MARY_JP
#elif defined(MARY_MFOMT_JP)
#define MARY_MFOMT
#define MARY_JP
#endif
mary_callable_table
{
    /* VM-internal stack/control slots, not source-level calls. */
    NULL,
    NULL,
    SetEntityPosition,
    GetEntityX,
    GetEntityY,
    SetEntityFacing,
    GetEntityFacing,
    SetEntitySeatState,
    MoveEntityXTo,
    MoveEntityXToRaw,
    MoveEntityYTo,
    MoveEntityYToRaw,
    WaitForEntityMovement,
    SetEntityAnim,
    NoOp014,
    HideEntity,
    SetEntityAuxRenderProfile,
    StartEntityEffect,
    StopEntityEffect,
    GetOppositeFacing,
    GetEntityLocation,
    OffsetEntityPosition,
    ChangeMap,
    PanCameraTo,
    WaitForCameraMovement,
    PlayBGM,
    StopBGM,
    PlaySong,
    StopAllSongs,
    FadeOutBGM,
    ResetTalkUi,
    TalkOpen,
#if defined(MARY_MFOMT)
    TalkOpenNoPortrait,
#endif
    TalkClose,
    TalkMessage,
    TalkMessageSlow,
    TalkAppendMessage,
    TalkPromptChoice2,
    TalkPromptChoice3,
    TalkPromptChoice4,
    TalkChoice2,
    TalkChoice3,
    TalkChoice4,
    TalkChoice5,
    TalkChoice6,
    SetTalkNameplateCharacter,
    SetTalkNameplateText,
    ClearTalkNameplate,
    SetTalkPortrait,
    ClearTalkPortrait,
    ShowTalkHeartIndicator,
    HideTalkHeartIndicator,
    FadeOutScreen,
    FadeInScreen,
    FadeInScreenAlias,
    WaitFrames,
    CallScript,
    SetTextVariableNumber,
    SetTextVariableNumberFieldWidth,
    SetTextVariableString,
    RandomU15,
    RandomIntInclusive,
    VarGet,
    VarSet,
    IsPlayerHoldingNothing,
    GetPlayerHeldItemKind,
    IsPlayerHeldItemWrapped,
    GetPlayerHeldFoodId,
    GetPlayerHeldArticleId,
    GetPlayerHeldChickenId,
    UsePlayerHeldItem,
    ClearPlayerHeldItem,
    SetPlayerHeldFood,
    SetPlayerHeldArticle,
    SetPlayerHeldWrappedFood,
    SetPlayerHeldWrappedArticle,
    CanDiscardPlayerHeldArticle,
    TryShipPlayerHeldItem,
    ThrowPlayerHeldItem,
    GetPlayerHeldToolId,
    GetPlayerHeldToolStackCount,
    SetPlayerHeldTool,
    ClearPlayerHeldTool,
    FindFoodInRucksack,
    FindArticleInRucksack,
    ClearRucksackItemSlot,
    GetFirstFreeRucksackToolSlot,
    GetFirstFreeRucksackItemSlot,
    AddArticleToRucksack,
    AddFoodToRucksack,
    AddToolToRucksack,
    ShowPlayerHoldingTool,
    ChangePlayerStaminaAndFatigue,
    IsPlayerHoldingTool,
    PlayerOwnsTool,
    PlayerOwnsFood,
    PlayerOwnsArticle,
    RemoveAllOwnedArticles,
    ObtainPowerBerry,
    ObtainMysticBerry,
    IsPlayerRidingHorse,
    EnterHotSpringBathingState,
    ExitHotSpringBathingState,
    PreservePlayerLocationForNextDay,
    ClearPreservedPlayerLocation,
    GetPreservedPlayerMapId,
    EatRandomMeal,
    PreparePlayerForScriptedAnimation,
    RestorePlayerAfterScriptedAnimation,
    GetPresentedItemKind,
    GetPresentedItemId,
    IsPresentedItemGiftWrapped,
    CanReceiveTool,
    CanReceiveFood,
    CanReceiveArticle,
    GivePlayerBasket,
    PlayerHasBasket,
    UpgradeRucksack,
    GetRucksackUpgradeLevel,
    SetPlayerActorUpdateSuspended,
#if defined(MARY_MFOMT)
    GetPlayerOutfitColor,
    SetPlayerOutfitColor,
#endif
    HasLocalLinkMilestone,
    HasReceivedLinkMilestone,
    SetLocalLinkMilestone,
    ClearLocalLinkMilestone,
    IsCharacterAtPlayerLocation,
    GetNpcFriendship,
    AddNpcFriendship,
    SetNpcFriendship,
    GetDaysSinceLastSpokenToNpc,
    MarkNpcSpokenTo,
    WasNpcSpokenToToday,
    WasNpcSpokenToJustNow,
    HasMetNpc,
    MarkNpcGifted,
    WasNpcGiftedToday,
    GetCharacterLove,
    AddCharacterLove,
    SetCharacterLove,
    SetEntityEventScript,
    ClearEntityEventScript,
    OpenSupermarketShop,
    PurchaseSupermarketItem,
    OpenWonShop,
    OpenCarpenterShop,
    OpenBlacksmithShop,
    OpenClinicShop,
    OpenBeachCafeShop,
    OpenYodelRanchShop,
    OpenWineryShop,
    OpenInnShop,
    OpenPoultryFarmShop,
    OpenSpecialMerchantShop,
    OpenGiftWrappingMenu,
    ShowReferencePage,
    OpenBookList,
    OpenLetterList,
    OpenCalendar,
    OpenShelf,
    OpenToolChest,
    OpenRefrigerator,
    OpenClock,
    OpenCookingMenu,
    OpenRecipeList,
    RunGameCubeLink,
    OpenNameEntry,
    StartFarmInheritanceFlashback,
    OpenNameEntryKeyboard,
    OpenRucksackMenu,
    SelectFestivalAnimal,
    FinishWeddingSequence,
    OpenFarmingTutorial,
    PrepareClockMenuTransition,
    RestoreAfterClockMenu,
    PrepareCookingMenuTransition,
    RestoreAfterCookingMenu,
    PrepareRecipeMenuTransition,
    RestoreAfterRecipeMenu,
    RecordPlayerHasAlbum,
    SwapRecordPlayerAlbum,
    RemoveRecordPlayerAlbum,
    LightFireplaceAtLocation,
    IsFireplaceLitAtLocation,
    SetVaseArticleId,
    GetVaseArticleId,
    IsChickenFeedTroughFilled,
    FillChickenFeedTrough,
    BeginEggIncubation,
    IsIncubatorOccupied,
    GetIncubatorCapacity,
    IsEggReadyToHatch,
    AttemptEggHatch,
    IsBarnFeedTroughFilled,
    FillBarnFeedTrough,
    StartShipmentBoxDepositAnimation,
    GetPregnancyStallCapacity,
    IsBarnAnimalReadyToGiveBirth,
    AttemptBarnAnimalBirth,
    BuildMountainCottage,
    BuildSeasideCottage,
    HasGoldenLumberOnFarm,
    OpenDoor,
    CloseDoor,
    CompleteRucksackUpgrade,
    CompleteBlueFeatherPurchase,
    GetHarvestSpriteCurrentTask,
    GetHarvestSpriteWorkDaysLeft,
    GetHarvestSpriteTaskExperience,
    HasHarvestSpritePlayedMinigameToday,
    HasHarvestSpriteTaskExperience,
    StartHarvestSpriteTask,
    StopHarvestSpriteWorkForToday,
    IsHarvestSpriteWorkComplete,
    RunHarvestSpriteAnimalCareMinigame,
    RunHarvestSpriteHarvestingMinigame,
    RunHarvestSpriteWateringMinigame,
    RunChickenFestivalContest,
    RunHorseRace,
    PrepareHorseRaceEntries,
    OpenHorseRaceMedalExchange,
    RunFrisbeeGame,
    RunFrisbeeTournamentRound,
    PrepareAnimalFestivalOpponents,
    GetTVShoppingSelection,
    GetPendingTVShoppingItem,
    IsTVShoppingDeliveryReady,
    SetTVShoppingSelection,
    ConfirmTVShoppingOrder,
    CompleteTVShoppingDelivery,
    IsVacationVillaBuilt,
    AreAllVillagersAtMaxFriendship,
    HasShippedOneOfEachCrop,
    AreAllFarmAnimalsAtMaxAffection,
    HasShippedOneOfEachMineral,
    HasCaughtEveryFishSpecies,
    GetTotalFishCaught,
    HasObtainedMythicTool,
    HasShippedOneOfEachProduct,
    GetMoney,
    AddMoney,
    SubtractMoney,
    EnableScriptedNpcControl,
    DisableScriptedNpcControl,
    GetBlacksmithOrderId,
    IsBlacksmithOrderReady,
    CollectBlacksmithOrder,
    SetGameTime,
    IsLetterWaiting,
    HasReceivedLetter,
    DeliverLetter,
    MarkLetterRead,
    GetWaitingLetterCount,
    GetSavedLetterCount,
    ShowTelevisionMessage,
    SetTelevisionProgram,
    EndTelevisionProgram,
    RefreshAllNpcSchedules,
    DoesAnimalExist,
    CreateFarmHorse,
    RemoveFarmHorse,
    GetAnimalName,
    GetInteractingAnimalIndex,
    HasAnimalBeenTalkedTo,
    SetAnimalTalkedTo,
    AddAnimalAffection,
    IsAnimalUnhappy,
    IsAnimalSick,
    IsAnimalPregnant,
    GetAnimalHealthyPregnancyDays,
    GetAnimalAge,
    GetAnimalAffection,
    GetAnimalGrowthStage,
    IsSheepSheared,
    CountAnimalsByLifeState,
    ShowLivestockNeglectDeathSummary,
    RemoveLivestockDeadFromNeglect,
    ShowNaturalLivestockDeathSummary,
    RemoveNaturallyDeadLivestock,
    IsCowAtBarnSlot,
    GetCowCount,
    GetSheepCount,
    GetChickenCount,
    SetContestAnimal,
    ClearContestAnimal,
    GetContestAnimalIndex,
    SetTextVariableToCaughtFishName,
    GetCaughtFishSize,
    IsCaughtFishMaximumSize,
    IsCaughtFishRiverKing,
    SelectMoonViewingPartner,
#if defined(MARY_MFOMT)
    SelectMoonViewingPartnerAlias,
#endif
    GetCookingFestivalDishRating,
    SelectThomasStockingGift,
    GetRandomSpouseGiftArticleId,
    UnlockNextVanAlbum,
    AreAllVanAlbumsAvailable,
    SetPlayerNicknameForSpouse,
    PlacePlayerAtFarmhouseBed,
    FlashScreenColor,
    RebuildMapEntitiesForNewDay,
    CureAllSickLivestock,
    AddAffectionToAllFarmAnimals,
    EnableShootingStarShippingBonus,
    GetAmountShipped,
    CreatePlayerChildEntity,
    RelocateEntityToMap,
    CreateNewYearSunriseEffect,
    PlayNewYearSunriseEffect,
    DestroyNewYearSunriseEffect,
    PlayStarSparkleEffect,
    GetKnownRecipeCount,
    GenerateMineFloorLayout,
    DescendMineFloor,
    IsToolCursed,
    AdvanceCursedToolLiftProgress,
    AttemptChurchCursedToolRemoval,
    CreateEventIcon,
    RemoveEventIcon,
    GetFoodIconId,
    GetArticleIconId,
    GetToolIconId,
    NoOpTutorialFieldTile,
    NoOpTutorialFieldObject,
    NoOpTutorialEggDefinition,
    NoOpTutorialEggSelection,
    BeginHoldingActorGraphic,
    NoOpAnimalEventEntityInitialization,
    SetTextVariableToProductName,
    GetEventContextValue,
    SelectNextCursedTool,
#if defined(MARY_MFOMT)
    IsChickenIncubatorOccupied,
    IsCowPregnancySlotOccupied,
    IsSheepPregnancySlotOccupied,
    GetFishCatchCount,
    GetLargestCaughtFishSize,
    IsMapRegistered,
    GetToolExperience,
    CountFestivalWinningAnimals,
#endif
};

/*
 * Mary-C callable declarations
 *
 * Declaration order exactly follows mary_callable_table and therefore mirrors
 * its logical callable ID order.
 *
 * Mary*Id and Mary*Kind accept symbols from mary_constants.mary.h or original
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
 * Mary*Id、Mary*Kind 可使用 mary_constants.mary.h 的符号或原始整数，
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
 * family. A numbered FuncXXX/ProcXXX name is retained until source, ROM handler,
 * and script context establish a stable contract.
 *
 * callable 命名约定
 *
 * 语义名称采用成熟 GBA 反编译工程常见的“动词在前”风格：存储值使用
 * Get/Set，条件判断使用 Is/Has/Can，状态修改使用 Add/Remove/Clear，界面表现
 * 使用 Show/Hide/Open/Close，音频使用 Play/Stop，多阶段操作使用
 * Start/Wait/Finish，运行时对象使用 Create/Destroy。同一功能域的关联函数
 * 共用相同名词，仅以操作动词区分，以便按功能族检索。只有源码、ROM handler
 * 与脚本上下文共同证明稳定契约后，才会将 FuncXXX/ProcXXX 替换为语义名。
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
 *
 * 设置运行时地图实体的位置与朝向。
 * 参数：entity_id 为运行时地图实体 ID；x、y 为地图坐标；facing 为引擎朝向值。
 */
void SetEntityPosition(MaryEntityId entity_id, MaryMapSpaceX x, MaryMapSpaceY y, MaryFacingDirection facing);

/*
 * Returns a runtime map entity's X coordinate.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the entity's map X coordinate.
 *
 * 返回运行时地图实体的 X 坐标。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体的地图 X 坐标。
 */
MaryMapSpaceX GetEntityX(MaryEntityId entity_id);

/*
 * Returns a runtime map entity's Y coordinate.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the entity's map Y coordinate.
 *
 * 返回运行时地图实体的 Y 坐标。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体的地图 Y 坐标。
 */
MaryMapSpaceY GetEntityY(MaryEntityId entity_id);

/*
 * Sets a runtime map entity's facing direction.
 * Parameters: entity_id is the runtime map entity ID; facing is the engine
 * direction value.
 *
 * 设置运行时地图实体的朝向。
 * 参数：entity_id 为运行时地图实体 ID；facing 为引擎朝向值。
 */
void SetEntityFacing(MaryEntityId entity_id, MaryFacingDirection facing);

/*
 * Returns a runtime map entity's facing direction.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the engine direction value.
 *
 * 返回运行时地图实体的朝向。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：引擎朝向值。
 */
MaryFacingDirection GetEntityFacing(MaryEntityId entity_id);

/*
 * Forces a runtime entity's seated-state phase.
 * Parameters: entity_id selects
 * the event entity through the scene's virtual entity lookup; seat_state is
 * written to the actor byte at offset 0x21. Vanilla tea-party scripts use state
 * 1 while seating the sprites and state 2 when releasing that arrangement.
 *
 * 强制设置运行时实体的就座状态阶段。
 * 参数：entity_id 会经场景虚函数查询选中
 * 事件实体；seat_state 写入角色偏移 0x21 的字节字段。原版茶会脚本在安排
 * 小矮人就座时使用状态 1，在解除该安排时使用状态 2。
 */
void SetEntitySeatState(MaryEntityId entity_id, MaryEntitySeatState seat_state);

/*
 * Starts horizontal movement toward an absolute X coordinate.
 * Parameters: entity_id selects the runtime scene entity; x is the target
 * coordinate; speed is a MaryEntityMoveSpeed measured in pixels per frame and
 * converted by the engine to Q16.16. The call starts movement and returns immediately; use
 * WaitForEntityMovement when synchronization is required.
 *
 * 启动实体向绝对 X 坐标的水平移动。
 * 参数：entity_id 为运行时场景实体编号；x 为目标坐标；speed 为以每帧像素数
 * 表示的 MaryEntityMoveSpeed，引擎会将其转换成 Q16.16。本调用只启动移动并立即返回；需要同步时应再调用
 * WaitForEntityMovement。
 */
void MoveEntityXTo(MaryEntityId entity_id, MaryMapSpaceX x, MaryEntityMoveSpeed speed);

/*
 * Starts horizontal movement like MoveEntityXTo.
 * Parameters: entity_id is the
 * runtime scene entity; x is the absolute target coordinate; speed_q16 is a
 * MaryEntityMoveSpeedQ16 passed directly without scaling.
 *
 * 与 MoveEntityXTo 相同地启动水平移动。
 * 参数：entity_id 为运行时场景实体；
 * x 为绝对目标坐标；speed_q16 为 MaryEntityMoveSpeedQ16，会直接传入而不做缩放。
 */
void MoveEntityXToRaw(MaryEntityId entity_id, MaryMapSpaceX x, MaryEntityMoveSpeedQ16 speed_q16);

/*
 * Starts vertical movement toward an absolute Y coordinate.
 * Parameters: entity_id selects the runtime scene entity; y is the target
 * coordinate; speed is a MaryEntityMoveSpeed measured in pixels per frame and
 * converted by the engine to Q16.16. Use WaitForEntityMovement when synchronization is required.
 *
 * 启动实体向绝对 Y 坐标的垂直移动。
 * 参数：entity_id 为运行时场景实体编号；y 为目标坐标；speed 为以每帧像素数
 * 表示的 MaryEntityMoveSpeed，引擎会将其转换成 Q16.16。需要同步时应再调用 WaitForEntityMovement。
 */
void MoveEntityYTo(MaryEntityId entity_id, MaryMapSpaceY y, MaryEntityMoveSpeed speed);

/*
 * Starts vertical movement like MoveEntityYTo.
 * Parameters: entity_id is the
 * runtime scene entity; y is the absolute target coordinate; speed_q16 is a
 * MaryEntityMoveSpeedQ16 passed directly without scaling.
 *
 * 与 MoveEntityYTo 相同地启动垂直移动。
 * 参数：entity_id 为运行时场景实体；
 * y 为绝对目标坐标；speed_q16 为 MaryEntityMoveSpeedQ16，会直接传入而不做缩放。
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
 * Sets a runtime map entity's animation.
 * Parameters: entity_id is the runtime map entity ID; animation_id is the
 * target-specific engine animation ID from the complete ANIMATION_ID_* domain.
 *
 * 设置运行时地图实体的动画。
 * 参数：entity_id 为运行时地图实体 ID；animation_id 使用目标版本完整的
 * ANIMATION_ID_* 引擎动画域。
 */
void SetEntityAnim(MaryEntityId entity_id, MaryAnimationId animation_id);

/*
 * Compatibility no-op at callable slot 0x00E. The VM consumes one integer,
 * then all four native FoMT/MFoMT US/JP implementations call an empty function
 * that immediately returns. It is exposed so otherwise-unused binary input
 * containing this slot can still round-trip without an unknown call.
 * Parameter: unused_value is preserved only as the observed stack operand;
 * the native no-op does not inspect it.
 *
 * callable 槽 0x00E 的兼容空操作。VM 会取出一个整数，随后 FoMT/MFoMT 的
 * US/JP 四版原生实现都会调用一个立即返回的空函数。公开它仅用于让包含该槽的
 * 二进制输入仍能无损往返，而不是建议新脚本使用。
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
 *
 * 选择临时事件实体使用的辅助渲染组件。entity_id 会经场景虚函数查询选中
 * 运行时实体；render_profile 原样写入其视觉控制器偏移 0x88。原生渲染器把
 * 0、1、2 分别映射到辅助组件 0、1、2，值 3 则跳过该组件。原版脚本分别将
 * 配置 0、2、3 用于小型动物、家畜和人物角色；原生实体构造器也会使用配置 1。
 */
void SetEntityAuxRenderProfile(MaryEntityId entity_id, MaryEntityAuxRenderProfile render_profile);

/*
 * Starts an auxiliary visual effect attached to a live scene entity.
 * Parameters: entity_id selects the entity; effect_id selects the engine
 * emote bubble; persistent is zero for the ordinary finite form and nonzero
 * for the persistent form. effect_id uses ENTITY_EMOTE_* or the exact numeric
 * resource index.
 *
 * 在运行时场景实体上启动一个附加视觉效果。
 * 参数：entity_id 选择实体；effect_id 使用 ENTITY_EMOTE_* 或精确数字资源
 * 索引来选择表情气泡；persistent 为零时使用普通有限时长形式，非零时使用
 * 持续形式。
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
 * Return value: the opposite engine direction value.
 *
 * 将朝向转换为相反方向。
 * 参数：facing 为引擎朝向值。
 * 返回值：相反的引擎朝向值。
 */
MaryFacingDirection GetOppositeFacing(MaryFacingDirection facing);

/*
 * Returns the map containing a runtime map entity. FoMT source calls this
 * value `location`, and all four ROM handlers return the same map-ID field
 * consumed by ChangeMap and the fireplace callables.
 * Parameter: entity_id is the runtime map entity ID.
 * Return value: the entity's MaryMapId.
 *
 * 返回运行时地图实体所在的地图。FoMT 源码将该值称为 `location`；四个 ROM
 * 的处理函数均返回与 ChangeMap 及壁炉函数共用的地图 ID 字段。
 * 参数：entity_id 为运行时地图实体 ID。
 * 返回值：实体所在地图的 MaryMapId。
 */
MaryMapId GetEntityLocation(MaryEntityId entity_id);

/*
 * Moves a live scene entity relative to its current position.
 * Parameters:
 * entity_id selects the runtime entity; delta_x and delta_y are signed
 * map-space offsets added to its current coordinates. This is distinct from
 * SetEntityPosition, whose coordinates are absolute. The MFoMT-US native
 * implementation at 0x0801232C resolves the entity, reads its current signed
 * X and Y fields at offsets 0x0A and 0x0E, adds delta_x and delta_y, and passes
 * the resulting coordinates to the entity-position updater. Matching FoMT/
 * MFoMT US/JP event scripts use it to shift wedding participants by one tile.
 *
 * 按当前位置相对移动运行时场景实体。
 * 参数：entity_id 选择运行时实体；delta_x、delta_y 是分别加到当前坐标上的
 * 有符号地图空间偏移量。本函数不同于使用绝对坐标的 SetEntityPosition。
 * MFoMT-US 位于 0x0801232C 的原生实现会解析实体，读取其偏移 0x0A、0x0E 的
 * 当前有符号 X、Y 字段，分别加上 delta_x、delta_y，再把结果交给实体位置更新
 * 函数。FoMT/MFoMT 的 US/JP 对应事件脚本都会用它把婚礼参与者平移一格。
 */
void OffsetEntityPosition(MaryEntityId entity_id, int delta_x, int delta_y);

/*
 * Changes the active field map and installs the destination coordinates in
 * the engine's packed location record.
 * Parameters: map_id is a MaryMapId or
 * the exact target-specific numeric map ID; x and y are destination map-space
 * coordinates. Scripts normally fade out first and then set the player
 * entity's position/facing explicitly after this call.
 *
 * 切换当前野外地图，并把目标坐标写入引擎的压缩位置记录。
 * 参数：map_id 为 MaryMapId 或目标版本的精确数字地图 ID；x、y 为目标地图
 * 坐标。脚本通常先淡出画面，调用本函数后再显式设置玩家实体的位置和朝向。
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
 *
 * 启动事件镜头向指定地图坐标平移。
 * 参数：x、y 为地图空间中的镜头目标；speed 为场景控制器采用的整数移动速度。
 * 原版脚本使用的值包括 1、2、5，因此它是连续速度量，而非封闭的选择枚举。
 * 本函数不会移动玩家实体。脚本经常另外启动实体 0 的对应移动，再让
 * 镜头以同一目标同步平移。后续指令必须等待镜头本身完成时，应调用
 * WaitForCameraMovement。
 */
void PanCameraTo(MaryMapSpaceX x, MaryMapSpaceY y, MaryCameraMoveSpeed speed);

/*
 * Suspends the current script until the field controller reports that the
 * active event-camera pan has finished. It does not wait for an entity's
 * movement; use WaitForEntityMovement for that independent state.
 *
 * 暂停当前脚本，直到场景控制器报告正在执行的事件镜头平移已经完成。它不会
 * 等待实体移动；后者是由 WaitForEntityMovement 独立等待的状态。
 */
void WaitForCameraMovement(void);

/*
 * Starts background music through the event state's dedicated BGM player.
 * Parameters: start_mode selects AUDIO_START, AUDIO_START_WEAK, or
 * AUDIO_START_OR_CONTINUE; song_id is the m4a song-table sequence ID.
 *
 * 通过事件状态中的专用 BGM 播放器播放背景音乐。
 * 参数：start_mode 选择 AUDIO_START、AUDIO_START_WEAK 或
 * AUDIO_START_OR_CONTINUE；song_id 为 m4a 曲目表中的序列 ID。
 */
void PlayBGM(MaryAudioStartMode start_mode, MaryAudioSequenceId song_id);

/*
 * Stops the current background music.
 * Parameters: none.
 *
 * 停止当前背景音乐。
 * 参数：无。
 */
void StopBGM(void);

/*
 * Starts a song or sound-effect sequence through an available event
 * MusicPlayer.
 * Parameters: start_mode selects AUDIO_START,
 * AUDIO_START_WEAK, or AUDIO_START_OR_CONTINUE; sequence_id is the m4a
 * song-table sequence ID.
 *
 * 通过事件可用的 MusicPlayer 播放歌曲或音效序列。
 * 参数：start_mode 选择 AUDIO_START、AUDIO_START_WEAK 或
 * AUDIO_START_OR_CONTINUE；sequence_id 为 m4a 曲目表中的序列 ID。
 */
void PlaySong(MaryAudioStartMode start_mode, MaryAudioSequenceId sequence_id);

/*
 * Stops all active songs or sound sequences.
 * Parameters: none.
 *
 * 停止所有正在播放的歌曲或声音序列。
 * 参数：无。
 */
void StopAllSongs(void);

/*
 * Fades out the current background music.
 * Parameters: none.
 *
 * 淡出当前背景音乐。
 * 参数：无。
 */
void FadeOutBGM(void);

/*
 * Destroys and clears the talk UI's auxiliary components and active text
 * window, then clears the corresponding active-state bit. This is a stronger
 * reset than TalkClose() and was not called by the four vanilla script sets.
 *
 * 销毁并清空对话界面的辅助组件及当前文本窗口，同时清除对应的活动状态位。
 * 本操作比 TalkClose() 更彻底，四套原版脚本均未调用它。
 */
void ResetTalkUi(void);

/*
 * Opens the dialogue window. Call this before TalkMessage or TalkChoiceN.
 * Parameters: none.
 *
 * 打开对话框。应在 TalkMessage 或 TalkChoiceN 之前调用。
 * 参数：无。
 */
void TalkOpen(void);

/*
 * Opens MFoMT's dialogue window without a portrait or named speaker. The
 * native handler first removes the existing portrait and speaker components,
 * then creates a fresh text-window component. Use this for narration and
 * other speakerless messages; ordinary character dialogue uses TalkOpen().
 * Parameters: none.
 *
 * 打开 MFoMT 的无头像、无说话者姓名对话框。原生处理函数会先移除现有头像与
 * 说话者组件，再创建新的文本窗口组件。旁白及其他无说话者文本使用本函数；
 * 普通人物对话使用 TalkOpen()。
 * 参数：无。
 */
void TalkOpenNoPortrait(void);

/*
 * Closes the dialogue window opened by TalkOpen.
 * Parameters: none.
 *
 * 关闭由 TalkOpen 打开的对话框。
 * 参数：无。
 */
void TalkClose(void);

/*
 * Displays a message in the open dialogue window.
 * Parameter: message is a text symbol from the current script's mary_text_table.
 *
 * 在已打开的对话框中显示消息。
 * 参数：message 为当前脚本 mary_text_table 中的文本符号。
 */
void TalkMessage(const char *message);

/*
 * Displays a message using the engine's slow-message mode.
 * Parameter: message is a text symbol from the current script's mary_text_table.
 *
 * 使用引擎的慢速文本模式显示消息。
 * 参数：message 为当前脚本 mary_text_table 中的文本符号。
 */
void TalkMessageSlow(const char *message);

/*
 * Appends text to the currently open dialogue without ending the composed
 * message.
 * Parameter: message is a text symbol from the current script. This
 * is used to build displays such as a date followed by a separately selected
 * time string.
 *
 * 向当前已打开的对话框追加文本，而不结束正在组合的消息。
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
 * separately to select the portrait and expression.
 *
 * 在对话框姓名牌中显示指定人物的本地化姓名。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确数字 ID。处理函数先经
 * 人物姓名表把 ID 转成字符串，再进入与 SetTalkNameplateText 相同的姓名牌
 * 渲染函数；它不会保存持久的说话者或交互人物身份。头像及表情需另外通过
 * SetTalkPortrait 设置。
 */
void SetTalkNameplateCharacter(MaryCharacterId character_id);

/*
 * Displays caller-supplied text in the talk-window nameplate. The native
 * renderer accepts at most 12 encoded bytes and clears the nameplate when the
 * text is empty or too long. Choice menus use this slot for short headings
 * such as "Pick one"; it can also provide a name not present in the character
 * table. The text remains active until replaced or cleared with
 * ClearTalkNameplate(). Parameter: text is a symbol from the current script.
 *
 * 在对话框姓名牌中显示调用者提供的文字。原生渲染器最多接受 12 个编码字节；
 * 空字符串或过长字符串会清除姓名牌。选择菜单用该槽显示“Pick one”等短标题，
 * 也可用它显示人物姓名表中没有的名字。文字会一直保留，直到被替换或由
 * ClearTalkNameplate() 清除。参数：text 为当前脚本文本表中的符号。
 */
void SetTalkNameplateText(const char *text);

/*
 * Clears the talk-window nameplate without closing the message window or
 * changing the portrait.
 *
 * 清除对话框姓名牌，但不关闭消息窗口，也不改变头像。
 */
void ClearTalkNameplate(void);

/*
 * Sets the portrait and expression shown by dialogue.
 * Parameter: portrait_id is TALK_PORTRAIT_* or the exact target-specific ID.
 * Use SetTalkNameplateCharacter separately to display the character's name.
 *
 * 设置对话显示的头像与表情。
 * 参数：portrait_id 为 TALK_PORTRAIT_* 或目标版本的精确 ID。
 * 人物姓名需另外通过 SetTalkNameplateCharacter 显示。
 */
void SetTalkPortrait(MaryTalkPortraitId portrait_id);

/*
 * Clears the portrait currently displayed by the talk UI. This does not close
 * the text box and does not change the independently rendered nameplate.
 *
 * 清除对话界面当前显示的头像。该操作不会关闭文本框，也不会改变独立渲染的
 * 姓名牌。
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
 * Native alias of FadeInScreen.
 * Parameters: fade_style and fade_speed use the
 * same domains as FadeInScreen. In MFoMT-US, callable slots 0x035 and 0x036
 * both pop the same two arguments and call 0x08012B08 with an identical final
 * mode value of zero; FoMT has the corresponding duplicate at slots 0x034 and
 * 0x035. The separate slot is preserved because scripts use both physical IDs.
 *
 * FadeInScreen 的原生别名。参数 fade_style 与 fade_speed 使用和
 * FadeInScreen 相同的编号域。MFoMT-US 的 callable 0x035 与 0x036 会弹出
 * 完全相同的两个参数，并以相同的末尾模式值 0 调用 0x08012B08；FoMT 对应的
 * 重复槽为 0x034 与 0x035。由于原版脚本确实会使用两个物理 ID，故保留独立槽位。
 */
void FadeInScreenAlias(MaryScreenFadeStyle fade_style, MaryScreenFadeSpeed fade_speed);

/*
 * Suspends the current script for frame_count engine frames.
 * Parameter: frame_count is written to the event state as an unsigned 16-bit
 * counter; values outside 0-65535 are truncated to their low 16 bits by the
 * native handler.
 *
 * 将当前脚本暂停 frame_count 个引擎帧。
 * 参数：frame_count 会作为无符号 16 位计数器写入事件状态；原生处理函数会把
 * 超出 0-65535 的值截成低 16 位。
 */
void WaitFrames(int frame_count);

/*
 * Calls another slot in the selected ROM's ordered script table.
 * Parameter: script_id is a SCRIPT_* symbol from mary_scripts.mary.h or the
 * exact numeric script-table ID. Both forms compile to the same integer.
 *
 * 调用所选 ROM 有序脚本表中的另一个槽位。
 * 参数：script_id 为 mary_scripts.mary.h 中的 SCRIPT_* 符号或精确数字脚本 ID；
 * 两种形式编译为同一个整数。
 */
void CallScript(MaryScriptId script_id);

/*
 * Writes a numeric value into a numbered text-substitution slot.
 * Parameters: variable_index selects the {VarN} destination and must be one
 * of TEXT_VARIABLE_1 through TEXT_VARIABLE_4; the native setter performs no
 * bounds check. value is the signed integer formatted by the text engine.
 *
 * 将数值写入编号文本替换槽。
 * 参数：variable_index 选择 {VarN} 目标，必须是 TEXT_VARIABLE_1 至
 * TEXT_VARIABLE_4；原生 setter 不做越界检查。value 为文本引擎格式化的
 * 有符号整数。
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
 * and truncation. The native formatter has a ten-digit work buffer, so new
 * scripts should keep field_width in the original 0-10 domain. variable_index
 * must be TEXT_VARIABLE_1 through TEXT_VARIABLE_4 because the native setter
 * does not bounds-check the destination. Shipped scripts
 * use width 2 for dates and width 3 for stock counts.
 *
 * 将十进制数字写入文本替换槽，并按指定字段宽度格式化。
 * 参数：variable_index 选择 {Var1}、{Var2} 等槽位；value 为要显示的有符号
 * 整数；field_width 非零时是精确十进制字段宽度。位数不足会在左侧补 ASCII
 * 空格，位数超出则截去高位、只保留指定数量的低位；宽度 0 表示不补齐也不
 * 截断。原生格式化器的工作缓冲区为十位，新脚本应维持原版 0-10 的宽度域。
 * variable_index 必须是 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4，因为原生 setter
 * 不检查目标槽是否越界。原版脚本中，日期使用宽度 2，库存数量使用宽度 3。
 */
void SetTextVariableNumberFieldWidth(
    MaryTextVariableSlot variable_index,
    int value,
    int field_width
);

/*
 * Writes a string into a numbered text-substitution slot.
 * Parameters: variable_index must be TEXT_VARIABLE_1 through TEXT_VARIABLE_4;
 * the native setter performs no bounds check. text is copied into the slot
 * and truncated to at most 22 encoded bytes before the terminating zero.
 * Calendar, clock, tool-name, and invitation scripts pass RIFF text symbols
 * here and later expand them through {Var1}-{Var4}. FOMT Studio's `Check_Date`
 * label is a type-recovery error: it renders the second text reference as an
 * integer table index. This callable is slot 0x03A in FoMT and 0x03B in MFoMT.
 *
 * 将字符串写入编号文本替换槽。
 * 参数：variable_index 必须是 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4，原生 setter
 * 不做越界检查。text 会复制到该槽，并在终止零字节前最多保留 22 个编码字节。
 * 日历、时钟、工具名称及邀请脚本会在此传入 RIFF 文本符号，随后通过
 * {Var1}-{Var4} 展开。FOMT Studio 的 `Check_Date` 标签源于类型恢复错误：它把
 * 第二个文本引用显示成了整数表索引。本 callable 在 FoMT 为 0x03A，在 MFoMT
 * 为 0x03B。
 */
void SetTextVariableString(MaryTextVariableSlot variable_index, const char *text);

/* Returns the engine PRNG's nonnegative 15-bit sample (0 through 32767).
 * It takes no arguments and performs no range reduction.
 *
 * 返回引擎伪随机数发生器的非负 15 位样本（0 至 32767）。本函数无参数，
 * 也不会执行区间缩放。
 */
int RandomU15(void);

/*
 * Generates a random integer in the inclusive range [min_value, max_value].
 * Parameters: min_value is the inclusive lower bound; max_value is the
 * inclusive upper bound. The native handler requires min_value <= max_value;
 * an inverted range skips random generation and does not produce a valid VM
 * result, so callers must not use it.
 * Return value: a generated integer inside the requested range.
 *
 * 生成闭区间 [min_value, max_value] 内的随机整数。
 * 参数：min_value 为包含在内的下界；max_value 为包含在内的上界。原生处理
 * 函数要求 min_value <= max_value；反向区间会跳过随机数生成，且不会产生
 * 有效的 VM 返回值，因此调用者不得传入反向区间。
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
 * integer stored in that variable.
 *
 * 写入编号游戏状态变量。
 * 参数：var_id 为 VAR_* 或精确数字变量 ID；value 为写入的整数。
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
 * Return value: one of the six HELD_ITEM_KIND_* values.
 * Use this result before interpreting a category-specific held-object ID.
 *
 * 返回玩家当前手持物的类别。
 * 参数：无。
 * 返回值：六种 HELD_ITEM_KIND_* 值之一。
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
 * Return value: FOOD_* or the original food ID; FOOD_NOT_PRESENT when the held
 * object is absent or is not food. The FoMT-US native wrapper checks both the
 * empty state and held category before extracting Food::GetId; either failed
 * check returns -1 to the VM.
 *
 * 返回当前手持食品的 ID。
 * 参数：无。
 * 返回值：FOOD_* 或原始食品 ID；当前对象不存在或不是食品时为
 * FOOD_NOT_PRESENT。FoMT-US 原生包装器会先检查空状态和手持类别，再读取
 * Food::GetId；任一检查失败都会向 VM 返回 -1。
 */
MaryFoodId GetPlayerHeldFoodId(void);

/*
 * Returns the article ID of the held object.
 * Parameters: none.
 * Return value: ARTICLE_* or the original article ID; ARTICLE_NOT_PRESENT when
 * the held object is absent or is not an article. The FoMT-US wrapper performs
 * both checks before Article::GetId and returns -1 on either failure.
 *
 * 返回当前手持物品的 ID。
 * 参数：无。
 * 返回值：ARTICLE_* 或原始物品 ID；当前对象不存在或不是物品时为
 * ARTICLE_NOT_PRESENT。FoMT-US 包装器在读取 Article::GetId 前执行两项检查，
 * 任一失败都会返回 -1。
 */
MaryArticleId GetPlayerHeldArticleId(void);

/*
 * Returns the chicken-record slot represented by the held object.
 * Parameters: none.
 * Return value: CHICKEN_SLOT_1 through CHICKEN_SLOT_8 for a held chicken, or
 * CHICKEN_SLOT_NONE when the held object is not a valid chicken record.
 *
 * 返回当前手持鸡所对应的鸡记录槽位。
 * 参数：无。
 * 返回值：手持有效鸡记录时为 CHICKEN_SLOT_1 至 CHICKEN_SLOT_8；当前手持物
 * 不是有效鸡记录时为 CHICKEN_SLOT_NONE。
 */
MaryChickenSlotIndex GetPlayerHeldChickenId(void);

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
 * Sets the player's held object to an unwrapped food.
 * Parameter: food_id is FOOD_* or the exact original food ID.
 *
 * 将玩家手持物设置为未包装食品。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 */
void SetPlayerHeldFood(MaryFoodId food_id);

/*
 * Sets the player's held object to an unwrapped article.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 *
 * 将玩家手持物设置为未包装物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 */
void SetPlayerHeldArticle(MaryArticleId article_id);

/*
 * Sets the player's held object to a gift-wrapped food.
 * Parameter: food_id is FOOD_* or the exact original food ID.
 *
 * 将玩家手持物设置为已包装食品。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 */
void SetPlayerHeldWrappedFood(MaryFoodId food_id);

/*
 * Sets the player's held object to a gift-wrapped article.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 *
 * 将玩家手持物设置为已包装物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 */
void SetPlayerHeldWrappedArticle(MaryArticleId article_id);

/*
 * Tests whether the currently held article may be discarded.
 * Parameters: none.
 * Return value: nonzero when discard is allowed; zero otherwise.
 *
 * 判断当前手持物品是否允许丢弃。
 * 参数：无。
 * 返回值：允许丢弃时为非零，否则为零。
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
 * Return value: TOOL_* or the original tool ID; TOOL_NOT_PRESENT when the held
 * tool stack is empty. The FoMT-US native wrapper initializes the result to
 * -1 and replaces it only after ToolStack::IsEmpty returns false; the retail
 * scripts use that sentinel before giving the fishing rod. MFoMT-US/JP call
 * sites use the same contract.
 *
 * 返回当前手持工具的 ID。
 * 参数：无。
 * 返回值：TOOL_* 或原始工具 ID；手持工具堆为空时为 TOOL_NOT_PRESENT。
 * FoMT-US 原生包装器先把结果初始化为 -1，仅在 ToolStack::IsEmpty 为假时才
 * 写入工具 ID；原版脚本会在赠送钓竿前使用该哨兵。MFoMT-US/JP 调用点使用
 * 相同约定。
 */
MaryToolId GetPlayerHeldToolId(void);

/*
 * Returns the number of copies in the player's held tool stack. This is an
 * inventory stack count, not a tool charge, upgrade level, or durability.
 * Parameters: none.
 * Return value: 1-99 for a nonempty stack, or
 * HELD_TOOL_STACK_NOT_PRESENT when no valid held tool stack exists.
 *
 * 返回玩家当前手持工具堆中的数量。该字段是背包堆叠数量，不是工具蓄力、
 * 升级等级或耐久度。
 * 参数：无。
 * 返回值：非空堆为 1-99；不存在有效手持工具堆时为
 * HELD_TOOL_STACK_NOT_PRESENT。
 */
MaryHeldToolStackCount GetPlayerHeldToolStackCount(void);

/*
 * Sets the player's held object to a tool.
 * Parameters: tool_id is TOOL_* or the exact original tool ID; stack_count is
 * the inventory quantity stored with that tool. The native ToolStack
 * constructor converts zero to one and clamps values greater than 99 to 99.
 *
 * 将玩家手持物设置为工具。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID；stack_count 为与工具一同保存的
 * 背包堆叠数量。原生 ToolStack 构造函数会把 0 转成 1，并把大于 99 的值
 * 限制为 99。
 */
void SetPlayerHeldTool(MaryToolId tool_id, int stack_count);

/*
 * Clears the player's currently held tool.
 * Parameters: none.
 *
 * 清除玩家当前手持工具。
 * 参数：无。
 */
void ClearPlayerHeldTool(void);

/*
 * Finds a food in the rucksack.
 * Parameter: food_id is FOOD_* or the exact original food ID.
 * Return value: the zero-based item slot, or RUCKSACK_SLOT_NOT_FOUND when the
 * requested food is absent.
 *
 * 在背包中查找食品。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 * 返回值：从零开始的物品槽位；没有该食品时返回 RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex FindFoodInRucksack(MaryFoodId food_id);

/*
 * Finds an article in the rucksack.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 * Return value: the zero-based item slot, or RUCKSACK_SLOT_NOT_FOUND when the
 * requested article is absent.
 *
 * 在背包中查找物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 * 返回值：从零开始的物品槽位；没有该物品时返回 RUCKSACK_SLOT_NOT_FOUND。
 */
MaryRucksackSlotIndex FindArticleInRucksack(MaryArticleId article_id);

/*
 * Clears one rucksack item slot.
 * Parameter: slot_id is the zero-based rucksack item-slot index and accepts a
 * MaryRucksackSlotIndex symbol or the identical raw integer.
 *
 * 清空一个背包物品槽。
 * 参数：slot_id 为从 0 开始的背包物品槽序号，可使用 MaryRucksackSlotIndex
 * 符号或数值相同的原始整数。
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
 * Parameters: article_id is ARTICLE_* or the exact original article ID;
 * requested_count is the number of copies to add. Each article occupies one
 * item slot.
 * Return value: the number of copies that could not be added. Zero means the
 * full request fit in the currently unlocked item slots.
 *
 * 尝试向背包加入一组物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID；requested_count 为请求
 * 加入的份数。每份物品占用一个物品槽。
 * 返回值：未能加入背包的剩余份数；返回 0 表示请求数量全部装入当前已解锁
 * 的物品槽。
 */
MaryUnaddedItemCount AddArticleToRucksack(MaryArticleId article_id, int requested_count);

/*
 * Attempts to add a food stack to the rucksack.
 * Parameters: food_id is FOOD_* or the exact original food ID;
 * requested_count is the number of copies to add. Each food occupies one item
 * slot.
 * Return value: the number of copies that could not be added. Zero means the
 * full request fit in the currently unlocked item slots.
 *
 * 尝试向背包加入一组食品。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID；requested_count 为请求加入的
 * 份数。每份食品占用一个物品槽。
 * 返回值：未能加入背包的剩余份数；返回 0 表示请求数量全部装入当前已解锁
 * 的物品槽。
 */
MaryUnaddedItemCount AddFoodToRucksack(MaryFoodId food_id, int requested_count);

/*
 * Attempts to add a quantity of one tool ID to the rucksack. Existing stacks
 * of the same tool are filled first and each tool stack holds at most 99.
 * Parameters: tool_id is TOOL_* or the exact original tool ID;
 * requested_count is the inventory quantity to add.
 * Return value: the quantity that could not be added. Zero means the full
 * request fit in the currently unlocked tool slots.
 *
 * 尝试向背包加入指定数量的同一种工具。引擎会先填充已有的同 ID 工具堆，
 * 每个工具堆最多保存 99 个。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID；requested_count 为请求加入的
 * 背包数量。
 * 返回值：未能加入背包的剩余数量；返回 0 表示请求数量全部装入当前已解锁
 * 的工具槽。
 */
MaryUnaddedItemCount AddToolToRucksack(MaryToolId tool_id, int requested_count);

/*
 * Makes the player actor visibly hold or present a tool.
 * Parameter: tool_id is TOOL_* or the exact original tool ID. This is used
 * when receiving tools, presenting the Blue Feather, showing a blessed tool,
 * and receiving the Goddess, Kappa, or Truth Gem.
 *
 * 让玩家角色以可见方式手持或展示工具。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID。获得工具、展示蓝色羽毛、
 * 展示解除诅咒后的工具，以及取得女神、河童或真实之玉时都会调用它。
 */
void ShowPlayerHoldingTool(MaryToolId tool_id);

/*
 * Applies signed changes to player stamina and fatigue.
 * Parameters: stamina_delta changes stamina; fatigue_delta changes fatigue.
 * Positive values increase and negative values decrease the corresponding
 * stat. All four vanilla script sets use the same contract for sleep recovery,
 * bathroom and fireplace effects, collapse treatment, and Doctor/Kai events.
 * This callable is physical slot 0x05B in FoMT and 0x05C in MFoMT because
 * MFoMT inserts ShowPlayerHoldingTool immediately before it.
 *
 * 对玩家体力与疲劳应用有符号变化量。
 * 参数：stamina_delta 改变体力；fatigue_delta 改变疲劳。
 * 正数提高对应数值，负数降低对应数值。四套原版脚本在睡眠恢复、浴室与壁炉
 * 效果、昏倒治疗以及 Doctor/Kai 事件中都使用相同参数约定。该 callable 在
 * FoMT 的物理槽为 0x05B；MFoMT 在它之前插入了 ShowPlayerHoldingTool，故其
 * 物理槽顺延为 0x05C。
 */
void ChangePlayerStaminaAndFatigue(int stamina_delta, int fatigue_delta);

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
 * Tests whether the player owns a specific tool.
 * Parameter: tool_id is TOOL_* or the exact original tool ID.
 * Return value: nonzero when owned; zero otherwise.
 *
 * 判断玩家是否拥有指定工具。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsTool(MaryToolId tool_id);

/* Tests whether the player owns a specific food in the held-item slot,
 * rucksack, or refrigerator.
 * Parameter: food_id is FOOD_* or the exact
 * original food ID. Return value: nonzero when owned; zero otherwise.
 *
 * 判断手持物槽、背包或冰箱中是否拥有指定食品。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsFood(MaryFoodId food_id);

/*
 * Tests whether the player owns a specific article.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 * Return value: nonzero when owned; zero otherwise.
 *
 * 判断玩家是否拥有指定物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 * 返回值：拥有时为非零，否则为零。
 */
MaryBool PlayerOwnsArticle(MaryArticleId article_id);

/*
 * Removes every owned copy of a specific article.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 *
 * 移除玩家拥有的指定物品的全部数量。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 */
void RemoveAllOwnedArticles(MaryArticleId article_id);

/*
 * Gives the player one Power Berry and runs the engine's acquisition action.
 * Parameters: none. This updates the Farmer power-berry state itself; event
 * flags that prevent an individual berry from being collected twice remain
 * the responsibility of the calling script.
 *
 * 给予玩家一枚力量果实，并执行引擎的取得动作。
 * 参数：无。
 * 本函数会直接更新 Farmer 的力量果实状态；用于防止某一枚果实被重复取得的
 * 事件标志，仍由调用脚本负责设置。
 */
void ObtainPowerBerry(void);

/*
 * Gives the player the Mystic Berry obtained from Kappa after offering ten
 * cucumbers, and runs the corresponding engine acquisition action.
 * Parameters: none. The calling event separately records that this unique
 * reward has been claimed.
 *
 * 给予玩家向 Kappa 供奉十根黄瓜后获得的神秘果实，并执行对应的引擎取得动作。
 * 参数：无。该唯一奖励是否已经领取，仍由调用事件另行记录。
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
 *
 * 返回当前“提交物品”事件所捕获物品的类别。女神供品脚本已证明会返回 FOOD
 * 与 ARTICLE。
 * 参数：无。
 */
MaryHeldItemKind GetPresentedItemKind(void);

/*
 * Returns the category-local ID paired with GetPresentedItemKind(). It is a
 * food ID when the kind is FOOD and an article ID when the kind is ARTICLE. A
 * single static return enum would therefore be misleading.
 * Parameters: none.
 *
 * 返回与 GetPresentedItemKind() 配套的类别内 ID。类别为 FOOD 时它是食品 ID，
 * 为 ARTICLE 时它是物品 ID，因此不能安全地声明为单一静态枚举类型。无参数。
 */
int GetPresentedItemId(void);

/*
 * Returns whether the item captured by GetPresentedItemKind() and
 * GetPresentedItemId() has the gift-wrap bonus. The offering scripts apply the
 * documented 25 percent relationship bonus when this value is nonzero.
 * Parameters: none.
 *
 * 返回 GetPresentedItemKind()/GetPresentedItemId() 捕获的物品是否带有礼物
 * 包装加成。供品脚本在本值非零时会应用 25% 的关系值加成。无参数。
 */
MaryBool IsPresentedItemGiftWrapped(void);

/* Tests whether the held-item slot, rucksack, or tool chest can accept a tool.
 * Parameter: tool_id is TOOL_* or the exact original tool ID.
 * Return value: nonzero when at least one applicable destination can accept
 * the tool; zero when all applicable destinations are full.
 *
 * 判断手持物槽、背包或工具箱能否容纳指定工具。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID。
 * 返回值：至少一个适用位置能够容纳时为非零；所有适用位置均已满时为零。
 */
MaryBool CanReceiveTool(MaryToolId tool_id);

/*
 * Tests whether a food item can be received without being lost.
 * Parameter: food_id is FOOD_* or the exact original food ID.
 * Return value: nonzero when the held-item slot, rucksack, or refrigerator can
 * accept the food; zero when all applicable destinations are full.
 * Shop scripts call this after the price check and before granting the item.
 *
 * 判断食品能否被玩家接收且不会丢失。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 * 返回值：手持物槽、背包或冰箱中至少一处能够容纳该食品时为非零；所有适用
 * 存放位置均已满时为零。商店脚本会在检查价格之后、交付商品之前调用它。
 */
MaryBool CanReceiveFood(MaryFoodId food_id);

/* Tests whether the held-item slot, rucksack, or shelf can accept an article.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 * Return value: nonzero when at least one applicable destination can accept
 * the article; zero when all applicable destinations are full.
 *
 * 判断手持物槽、背包或置物棚能否容纳指定物品。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 * 返回值：至少一个适用位置能够容纳时为非零；所有适用位置均已满时为零。
 */
MaryBool CanReceiveArticle(MaryArticleId article_id);

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
 *
 * 返回当前背包容量升级等级。
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
#if defined(MARY_MFOMT)
/*
 * Returns the female protagonist's current outfit color.
 * Parameters: none.
 *
 * 返回女主角当前的服装颜色。无参数。
 */
MaryOutfitColor GetPlayerOutfitColor(void);

/*
 * Changes the female protagonist's outfit color.
 * Parameters: color is an OUTFIT_COLOR_* value.
 *
 * 更改女主角的服装颜色。参数 color 为 OUTFIT_COLOR_*。
 */
void SetPlayerOutfitColor(MaryOutfitColor color);
#endif
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
 * Tests whether a character's scheduled location matches the player's current
 * location. The engine resolves character_id through the target's character
 * schedule table and compares the location field of the result.
 * Parameter: character_id is CHARACTER_* or the exact target-specific ID.
 * Return value: nonzero when both locations match; zero otherwise.
 *
 * 判断指定人物按日程计算出的当前位置是否与玩家当前位置相同。引擎通过目标
 * 版本的人物日程表解析 character_id，再比较结果中的地点字段。
 * 参数：character_id 为 CHARACTER_* 或目标版本的精确人物 ID。
 * 返回值：地点相同时为非零，否则为零。
 */
MaryBool IsCharacterAtPlayerLocation(MaryCharacterId character_id);

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
int GetNpcFriendship(MaryCharacterId character_id);

/*
 * Adds a signed amount to an NPC's friendship points.
 * Parameters: character_id selects the NPC; amount is the signed adjustment.
 * This is the modifying counterpart of GetNpcFriendship.
 *
 * 对 NPC 的友好度点数增加一个有符号数值。
 * 参数：character_id 选择 NPC；amount 为有符号调整量。
 * 本函数是 GetNpcFriendship 对应的修改操作。
 */
void AddNpcFriendship(MaryCharacterId character_id, int amount);

/*
 * Replaces an NPC's friendship value.
 * Parameters: character_id selects the NPC; friendship is the new value.
 *
 * 直接替换 NPC 的友好度数值。
 * 参数：character_id 选择 NPC；friendship 为新的友好度数值。
 */
void SetNpcFriendship(MaryCharacterId character_id, int friendship);

/*
 * Gets the number of days since the player last spoke to an NPC.
 * Parameter: character_id selects the NPC.
 * Return value: the stored day count, or zero when the ID is invalid.
 *
 * 获取玩家上次与某 NPC 交谈后经过的天数。
 * 参数：character_id 选择 NPC。
 * 返回值：保存的天数；ID 无效时返回零。
 */
int GetDaysSinceLastSpokenToNpc(MaryCharacterId character_id);

/*
 * Records that the player has just spoken to an NPC.
 * Parameter: character_id selects the NPC. This updates the state queried by
 * WasNpcSpokenToToday and WasNpcSpokenToJustNow.
 *
 * 记录玩家刚刚与某 NPC 交谈。
 * 参数：character_id 选择 NPC。本操作会更新 WasNpcSpokenToToday 与
 * WasNpcSpokenToJustNow 查询的状态。
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
 * Records that an NPC has received a gift in the current interaction.
 * Parameter: character_id selects the NPC. This updates the state queried by
 * WasNpcGiftedToday.
 *
 * 记录某 NPC 在当前交互中已经收到礼物。
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
int GetCharacterLove(MaryCharacterId character_id);

/*
 * Adds a signed amount to a romance candidate's love points.
 * Parameters: character_id selects the target-specific candidate; amount is
 * the signed adjustment. This is the modifying counterpart of GetCharacterLove.
 *
 * 对恋爱候选人的爱情度点数增加一个有符号数值。
 * 参数：character_id 选择目标版本中的恋爱候选人；amount 为有符号调整量。
 * 本函数是 GetCharacterLove 对应的修改操作。
 */
void AddCharacterLove(MaryCharacterId character_id, int amount);

/*
 * Replaces a romance candidate's love points with an exact value.
 * Parameters: character_id selects the target-specific candidate; love is the
 * new absolute value. Unlike AddCharacterLove(), this does not apply a delta.
 * Invalid or non-romance character IDs leave the candidate table unchanged.
 *
 * 将恋爱候选人的爱情度直接替换为指定值。
 * 参数：character_id 选择目标版本中的恋爱候选人；love 为新的绝对值。与
 * AddCharacterLove() 不同，本函数不会把参数当作增量。无效或非恋爱候选人的
 * ID 不会修改候选人表。
 */
void SetCharacterLove(MaryCharacterId character_id, int love);

/*
 * Assigns an event script to a live scene entity.
 * Parameters: entity_id selects the scene entity; script_id is a symbol from
 * mary_scripts.mary.h or the exact target-specific script-table ID.
 * The entity ID is deliberately untyped because it addresses live scene
 * entities rather than the character-name table.
 *
 * 为当前场景中的实体绑定事件脚本。
 * 参数：entity_id 选择场景实体；script_id 为 mary_scripts.mary.h 中的符号或
 * 目标版本脚本表的精确 ID。实体 ID 指向运行时场景实体，并非人物名称表编号，
 * 因此有意保留为普通 int。
 */
void SetEntityEventScript(MaryEntityId entity_id, MaryScriptId script_id);

/*
 * Clears the event script assigned to a live scene entity.
 * Parameter: entity_id selects the same scene-entity domain used by
 * SetEntityEventScript.
 *
 * 清除当前场景实体已绑定的事件脚本。
 * 参数：entity_id 与 SetEntityEventScript 使用同一个运行时场景实体编号域。
 */
void ClearEntityEventScript(MaryEntityId entity_id);

/*
 * Opens the supermarket's main shopping interface and waits until it closes.
 * Individual staple purchases are completed by PurchaseSupermarketItem.
 *
 * 打开杂货店主购物界面并等待其关闭。具体常备商品的购买由
 * PurchaseSupermarketItem 完成。
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
 */
void PurchaseSupermarketItem(MarySupermarketItemId item_id);

/*
 * Opens Won's merchant interface and waits until it closes.
 *
 * 打开 Won 的商店界面并等待其关闭。
 */
void OpenWonShop(void);

/*
 * Opens Gotz's carpenter and farm-upgrade interface and waits until it closes.
 *
 * 打开 Gotz 的木工与农场升级界面并等待其关闭。
 */
void OpenCarpenterShop(void);

/*
 * Opens Saibara's blacksmith interface and waits until it closes.
 *
 * 打开 Saibara 的锻冶屋界面并等待其关闭。
 */
void OpenBlacksmithShop(void);

/*
 * Opens the clinic's examination and medicine interface and waits until it
 * closes.
 *
 * 打开诊所的诊察与药品界面并等待其关闭。
 */
void OpenClinicShop(void);

/*
 * Opens Kai's seasonal beach cafe interface and waits until it closes.
 *
 * 打开 Kai 的夏季海之家商店界面并等待其关闭。
 */
void OpenBeachCafeShop(void);

/*
 * Opens Barley's Yodel Ranch shopping interface and waits until it closes.
 *
 * 打开 Barley 的 Yodel Ranch 商店界面并等待其关闭。
 */
void OpenYodelRanchShop(void);

/*
 * Opens Manna's winery shopping interface and waits until it closes.
 *
 * 打开 Manna 的果树园商店界面并等待其关闭。
 */
void OpenWineryShop(void);

/*
 * Opens Doug's inn food-ordering interface and waits until it closes.
 *
 * 打开 Doug 的旅馆点餐界面并等待其关闭。
 */
void OpenInnShop(void);

/*
 * Opens Lillia's poultry-farm shopping interface and waits until it closes.
 *
 * 打开 Lillia 的养鸡场商店界面并等待其关闭。
 */
void OpenPoultryFarmShop(void);

/*
 * Opens the visiting special merchant's shopping interface and waits until it
 * closes. This neutral name is intentional because the localized character
 * label at the same ID differs between targets.
 *
 * 打开来访特殊商人的购物界面并等待其关闭。由于同一人物 ID 的本地化名称在
 * 不同目标间存在差异，此处有意使用中性的功能名称。
 */
void OpenSpecialMerchantShop(void);

/*
 * Opens the supermarket gift-wrapping item-selection interface and waits until
 * it closes. The surrounding script performs the 100G availability check.
 *
 * 打开杂货店礼物包装的物品选择界面并等待其关闭。外围脚本负责检查是否有
 * 足够的 100G。
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
 *
 * 在游戏的模态页面查看器中打开指定编号的资料页。page_id 直接索引所选 ROM 的
 * 顶层资料页指针表：FoMT 有 137 个普通条目（0..136），MFoMT 有 190 个
 * （0..189）。每个条目再指向一个以空指针结束的文本指针列表；教程、书籍、
 * 电话簿、信件、通知及成就页实际共用这张物理表。同一游戏族的 US/JP 使用相同
 * 索引范围，但字符串内容仍属于各自目标。原生查看器还识别 0x1000..0x1002
 * 三个内部伪页面值；原版事件脚本不会在这里传入这些值。MaryReferencePageId
 * 已分别建模四版共用前缀和按男女版变化的后半段。
 */
void ShowReferencePage(MaryReferencePageId page_id);

/*
 * Opens the farmhouse bookshelf's collected-book list and waits until it
 * closes.
 *
 * 打开自宅书架的藏书列表并等待其关闭。
 */
void OpenBookList(void);

/*
 * Opens the farmhouse bookshelf's received-letter list and waits until it
 * closes. Scripts check that at least one letter exists before calling it.
 *
 * 打开自宅书架的收信列表并等待其关闭。脚本会在调用前检查至少存在一封信。
 */
void OpenLetterList(void);

/*
 * Opens the farmhouse calendar interface and waits until it closes. The same
 * callable is used by the calendar entity in every farmhouse upgrade stage.
 *
 * 打开自宅日历界面并等待其关闭。各个自宅扩建阶段的日历实体共用本函数。
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
 *
 * 打开自宅时钟界面并等待其关闭。各个自宅扩建阶段的时钟实体共用本函数。
 */
void OpenClock(void);

/*
 * Opens the farmhouse kitchen's cooking interface and waits until it closes.
 *
 * 打开自宅厨房的料理界面并等待其关闭。
 */
void OpenCookingMenu(void);

/*
 * Opens the player's learned-recipe list and waits until it closes.
 *
 * 打开玩家已经学会的菜谱列表并等待其关闭。
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
 *
 * 运行 Game Boy Advance/GameCube 通信界面，并等待终态
 * MaryGameCubeLinkResult。四版 VM 包装层使用相同的四类结果。
 * SUCCESS 是唯一会让原版女神脚本导入联机里程碑的结果；
 * CANCELED_OR_FAILED 覆盖普通失败及玩家／联机对方取消路径；
 * INCOMPATIBLE_SAVE_DATA 由显示两个游戏存档数据不兼容提示的原生
 * 状态选中；UNAVAILABLE 是没有已结束任务对象时的包装层后备值。
 * 返回值：上述 GAMECUBE_LINK_RESULT_* 之一。
 */
MaryGameCubeLinkResult RunGameCubeLink(void);

/*
 * Opens the name-entry interface for the selected target kind.
 * Parameter:
 * kind is NAME_ENTRY_* or an exact numeric kind; target_index selects the
 * animal slot for animal births and is zero for the horse, child, and custom
 * spouse-nickname forms. The call completes after the entered name has been
 * stored by the engine.
 *
 * 为指定目标类型打开命名输入界面。参数 kind 为 NAME_ENTRY_* 或精确数字类型；
 * target_index 在动物出生时选择动物槽位，马、孩子及自定义配偶昵称形式使用 0。
 * 调用会在引擎保存输入名称后完成。
 */
void OpenNameEntry(MaryNameEntryKind kind, int target_index);

/*
 * Starts the farm-inheritance flashback shown during Thomas's opening
 * explanation. FoMT-US/JP use this callable in the opening scripts. The
 * corresponding MFoMT slot is unused by the vanilla scripts, but its native
 * constructor is instruction-for-instruction equivalent and selects the same
 * region-specific scene state (US 0x29, JP 0x28), so it is the same retained
 * interface rather than an unidentified modal screen.
 *
 * 启动 Thomas 在开场说明牧场继承经过时使用的回忆场景。FoMT-US/JP
 * 的开场脚本会调用它。MFoMT 原版脚本虽未引用对应槽，但其原生构造器
 * 与 FoMT 指令级等价，且选择相同的地区场景状态（US 0x29、JP 0x28），
 * 因此这是保留的同一接口，而非无法识别的模态界面。
 */
void StartFarmInheritanceFlashback(void);

/*
 * Opens the reusable on-screen keyboard used by the naming sequence and waits
 * for editing to finish. The MFoMT-US/JP task implementations create a
 * 31-byte edit buffer, character-selection state machine, and commit the
 * resulting string back to the naming workflow. FoMT exposes the same stage at
 * raw callable slot 0x0A5; MFoMT uses 0x0A8.
 *
 * 打开命名流程复用的屏幕键盘，并等待编辑完成。MFoMT-US/JP 的任务实现会创建
 * 31 字节编辑缓冲区和字符选择状态机，最后把结果字符串写回命名流程。FoMT 的
 * 原始 callable 槽为 0x0A5，MFoMT 为 0x0A8。
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
 *
 * 打开玩家的完整背包界面并等待关闭。ROM 代码会分别构造“Tools”和“Items”
 * 面板、光标和物品交换控制；外围系统脚本随后检查使用飞行石等背包操作是否改变
 * 了当前地图。
 *
 * unused_stack_value 只用于保留原始字节码操作数（原版传入 1）。四个 native
 * handler 均不消费该值，背包任务也不会读取它，因此不能将其解释成菜单模式或
 * 布尔选项。
 */
void OpenRucksackMenu(int unused_stack_value);

/*
 * Opens the festival-entry selector for one livestock family.
 * Parameter:
 * festival_kind is FESTIVAL_ANIMAL_*; return value is the selected zero-based
 * animal slot, or -1 when selection is cancelled.
 *
 * 打开指定家畜类别的祭典参赛选择界面。参数 festival_kind 为
 * FESTIVAL_ANIMAL_*；返回所选动物从 0 开始的槽位，取消选择时返回 -1。
 */
MaryAnimalSlotIndex SelectFestivalAnimal(MaryFestivalAnimalKind festival_kind);

/*
 * Finishes the wedding presentation and prepares the transition into the
 * post-wedding nickname/new-life scene. All four targets call it after the
 * wedding fade-out; the FoMT library labels the corresponding raw slot as the
 * wedding-sequence finisher, and MFoMT implements the same task position.
 *
 * 结束婚礼表现并准备切换到婚后称呼／新生活场景。四个目标都在婚礼渐隐后调用；
 * FoMT 库把对应原始槽标为婚礼流程结束器，MFoMT 也在同一任务位置实现该操作。
 */
void FinishWeddingSequence(void);

/*
 * Opens one of Thomas's interactive farming tutorials and waits until it
 * closes.
 * Parameter: tutorial_kind is FARMING_TUTORIAL_*.
 *
 * 打开 Thomas 提供的一项交互式农场教程并等待其关闭。参数 tutorial_kind 为
 * FARMING_TUTORIAL_*。
 */
void OpenFarmingTutorial(MaryFarmingTutorialKind tutorial_kind);

/*
 * Runs the clock-menu preparation hook before fade-out and again after the
 * modal clock closes. Parameters: none. It is distinct from the cooking and
 * recipe hooks in all four native virtual-method tables.
 *
 * 在渐隐前以及时钟模态界面关闭后运行时钟界面准备钩子。
 * 参数：无。四个版本的原生虚函数表均将其与料理、菜谱钩子分开保存。
 */
void PrepareClockMenuTransition(void);

/*
 * Runs the clock-menu restoration hook after the field view is restored.
 * Parameters: none. In all four ROMs this invokes virtual callback 0x84 on
 * the active scene controller, paired with the separate clock preparation
 * path. It does not itself decode input, draw the clock, or reopen the menu.
 *
 * 场景画面恢复后运行时钟界面恢复钩子。
 * 参数：无。四个 ROM 均会在当前场景控制器上调用虚函数表偏移 0x84 的回调，
 * 与独立的时钟准备流程配对。本函数本身不解析输入、不绘制时钟，也不重新打开菜单。
 */
void RestoreAfterClockMenu(void);

/*
 * Runs the cooking-menu preparation hook before fade-out and again after the
 * modal cooking interface closes. Parameters: none. All four ROMs dispatch
 * virtual callback 0x88 on the active scene controller.
 *
 * 在渐隐前以及料理模态界面关闭后运行料理界面准备钩子。
 * 参数：无。四个 ROM 均调用当前场景控制器虚函数表偏移 0x88 的回调。
 */
void PrepareCookingMenuTransition(void);

/*
 * Runs the cooking-menu restoration hook after the field view is restored.
 * Parameters: none. All four ROMs dispatch virtual callback 0x8C, paired
 * with PrepareCookingMenuTransition at callback 0x88.
 *
 * 场景画面恢复后运行料理界面恢复钩子。
 * 参数：无。四个 ROM 均调用虚函数表偏移 0x8C 的回调，与偏移 0x88 的
 * PrepareCookingMenuTransition 配对。
 */
void RestoreAfterCookingMenu(void);

/*
 * Runs the recipe-list preparation hook before fade-out and again after the
 * modal recipe list closes. Parameters: none. All four ROMs dispatch virtual
 * callback 0x90 on the active scene controller.
 *
 * 在渐隐前以及菜谱列表模态界面关闭后运行菜谱界面准备钩子。
 * 参数：无。四个 ROM 均调用当前场景控制器虚函数表偏移 0x90 的回调。
 */
void PrepareRecipeMenuTransition(void);

/*
 * Runs the recipe-list restoration hook after the field view is restored.
 * Parameters: none. All four ROMs dispatch virtual callback 0x94, paired
 * with PrepareRecipeMenuTransition at callback 0x90.
 *
 * 场景画面恢复后运行菜谱列表恢复钩子。
 * 参数：无。四个 ROM 均调用虚函数表偏移 0x94 的回调，与偏移 0x90 的
 * PrepareRecipeMenuTransition 配对。
 */
void RestoreAfterRecipeMenu(void);

/*
 * Tests whether the farmhouse record player currently has an album available.
 * Parameters: none.
 * Return value: nonzero when an album is present; zero otherwise.
 * Record-player interaction scripts use this before selecting or playing music.
 *
 * 判断农舍唱片机当前是否有可用唱片。
 * 参数：无。
 * 返回值：存在唱片时为非零，否则为零。唱片机交互脚本会在选择或播放音乐前
 * 调用本函数。
 */
MaryBool RecordPlayerHasAlbum(void);

/*
 * Inserts an album article into the farmhouse record player and returns the
 * article ID of the album that was ejected. If the supplied article is not an
 * album, no album is inserted and the returned article slot is empty.
 * Parameter: article_id is the held article ID, normally one of Album 1-15.
 * Return value: the ejected album article ID, or the engine's empty-slot value.
 * The interaction script removes the held article first, then gives this
 * returned article back to the player.
 *
 * 将一张唱片类物品放入农舍唱片机，并返回被替换出来的旧唱片物品 ID。
 * 如果传入的物品不是唱片，则不会放入唱片，返回的物品槽为空。
 * 参数：article_id 为手持物品 ID，正常取值为唱片 1 至唱片 15。
 * 返回值：弹出的旧唱片物品 ID，或引擎使用的空物品槽值。交互脚本会先移除
 * 当前手持物品，再把本函数返回的旧唱片交还给玩家。
 */
MaryArticleId SwapRecordPlayerAlbum(MaryArticleId article_id);

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
MaryArticleId RemoveRecordPlayerAlbum(void);

/*
 * Lights the fireplace associated with a map. The FoMT source and all four
 * ROM wrappers consume the same map-ID domain used by ChangeMap and returned
 * by GetEntityLocation.
 * Parameter: map_id is the fireplace map's MaryMapId.
 *
 * 点燃指定地图关联的壁炉。FoMT 源码与四个 ROM 包装函数均使用 ChangeMap
 * 所接收、GetEntityLocation 所返回的同一地图 ID 域。
 * 参数：map_id 为壁炉所在地图的 MaryMapId。
 */
void LightFireplaceAtLocation(MaryMapId map_id);

/*
 * Tests whether the fireplace associated with a map is lit.
 * Parameter: map_id uses the same MaryMapId domain as
 * LightFireplaceAtLocation.
 * Return value: nonzero when lit; zero when unlit or when the location has no
 * supported fireplace state.
 *
 * 判断指定地图关联的壁炉是否已经点燃。
 * 参数：map_id 与 LightFireplaceAtLocation 使用同一 MaryMapId 编号域。
 * 返回值：已点燃时为非零；未点燃或地点不支持壁炉状态时为零。
 */
MaryBool IsFireplaceLitAtLocation(MaryMapId map_id);

/*
 * Places an article in the farmhouse vase and initializes its vase lifespan.
 * Parameter: article_id is normally one of ARTICLE_FLOWER_*; passing the exact
 * integer remains supported for byte-identical compilation. Vase interaction
 * scripts obtain this value from the held article and pair this operation with
 * GetVaseArticleId.
 *
 * 将物品放入农舍花瓶，并初始化该物品在花瓶中的保存期限。
 * 参数：article_id 通常为 ARTICLE_FLOWER_*；也可继续传入精确整数，编译字节
 * 保持一致。花瓶交互脚本从手持物品取得该值，并与 GetVaseArticleId 配合使用。
 */
void SetVaseArticleId(MaryArticleId article_id);

/*
 * Gets the article currently placed in the farmhouse vase.
 * Parameters: none.
 * Return value: ARTICLE_* or the exact article ID; ARTICLE_NOT_PRESENT when
 * the vase is empty. The FoMT-US native wrapper compares FarmHouse's physical
 * ARTICLE_NONE value and maps it to -1 before returning to the VM; the MFoMT
 * wrapper and both regional script sets use the same script-facing sentinel.
 *
 * 获取农舍花瓶中当前放置的物品。
 * 参数：无。
 * 返回值：ARTICLE_* 或精确物品 ID；花瓶为空时返回 ARTICLE_NOT_PRESENT。
 * FoMT-US 原生包装器会比较 FarmHouse 的物理 ARTICLE_NONE，并在返回 VM 前
 * 映射成 -1；MFoMT 包装器及两个地区的脚本使用同一个脚本侧哨兵。
 */
MaryArticleId GetVaseArticleId(void);

/*
 * Tests whether a chicken-coop feed trough is already filled.
 * Parameter: trough_index uses CHICKEN_FEED_TROUGH_*; only 1-4 are available
 * before the coop upgrade and all eight afterwards.
 * Return value: nonzero when feed is present; zero when the trough is empty.
 * Feed-box inspection scripts pair this with FillChickenFeedTrough.
 *
 * 判断鸡舍中的指定饲料槽是否已经放入饲料。
 * 参数：trough_index 使用 CHICKEN_FEED_TROUGH_*；扩建前仅 1-4 号可用，
 * 扩建后八个全部可用。
 * 返回值：已有饲料时为非零，饲料槽为空时为零。饲料箱检查脚本会将其与
 * FillChickenFeedTrough 配对使用。
 */
MaryBool IsChickenFeedTroughFilled(MaryChickenFeedTroughIndex trough_index);

/*
 * Fills a chicken-coop feed trough.
 * Parameter: trough_index uses CHICKEN_FEED_TROUGH_* and is checked against
 * the current four- or eight-trough capacity.
 * Call IsChickenFeedTroughFilled first when the script must avoid replacing
 * feed that is already present.
 *
 * 向鸡舍中的指定饲料槽放入饲料。
 * 参数：trough_index 使用 CHICKEN_FEED_TROUGH_*，并按当前四个或八个槽的
 * 容量校验。若脚本需要避免覆盖已有饲料，应先调用 IsChickenFeedTroughFilled。
 */
void FillChickenFeedTrough(MaryChickenFeedTroughIndex trough_index);

/*
 * Begins egg incubation in the selected coop incubator.
 * Parameter: incubator_index uses CHICKEN_INCUBATOR_1 or
 * CHICKEN_INCUBATOR_2. Only the first exists before the coop upgrade.
 * Check IsIncubatorOccupied before calling when replacement is not intended.
 *
 * 在选定的鸡舍孵化箱中开始孵化鸡蛋。
 * 参数：incubator_index 使用 CHICKEN_INCUBATOR_1 或
 * CHICKEN_INCUBATOR_2；扩建前只有第一个存在。不希望覆盖时，应先调用
 * IsIncubatorOccupied 检查。
 */
void BeginEggIncubation(MaryChickenIncubatorIndex incubator_index);

/*
 * Tests whether a chicken-coop incubator is occupied.
 * Parameter: incubator_index uses CHICKEN_INCUBATOR_*.
 * Return value: nonzero when occupied; zero when available.
 * Incubator inspection scripts use indices 0 and 1.
 *
 * 判断鸡舍中的指定孵化器是否已被占用。
 * 参数：incubator_index 使用 CHICKEN_INCUBATOR_*。
 * 返回值：已占用时为非零，可用时为零。孵化器检查脚本使用索引 0 和 1。
 */
MaryBool IsIncubatorOccupied(MaryChickenIncubatorIndex incubator_index);

/*
 * Gets the number of usable incubators in the current chicken coop.
 * Parameters: none.
 * Return value: 1 before the coop upgrade, 2 after the upgrade.
 *
 * 获取当前鸡舍中可用的孵化箱数量。
 * 参数：无。
 * 返回值：鸡舍扩建前为 1，扩建后为 2。
 */
int GetIncubatorCapacity(void);

/*
 * Tests whether the egg in an incubator has reached its hatch day.
 * Parameter: incubator_index uses CHICKEN_INCUBATOR_* and is checked against
 * the current one- or two-incubator capacity.
 * Return value: nonzero when ready to hatch; zero otherwise.
 *
 * 判断指定孵化箱中的鸡蛋是否已经到达孵化日。
 * 参数：incubator_index 使用 CHICKEN_INCUBATOR_*，并按当前一个或两个
 * 孵化箱的容量校验。
 * 返回值：可以孵化时为非零，否则为零。
 */
MaryBool IsEggReadyToHatch(MaryChickenIncubatorIndex incubator_index);

/*
 * Hatches a ready egg and inserts the new chick into the chicken-coop roster.
 * Parameter: incubator_index uses CHICKEN_INCUBATOR_* and is checked against
 * the current one- or two-incubator capacity.
 * Return value: the new chicken's roster slot, or -1 if hatching fails.
 *
 * 孵化已经到期的鸡蛋，并将新生小鸡加入鸡舍动物列表。
 * 参数：incubator_index 使用 CHICKEN_INCUBATOR_*，并按当前一个或两个
 * 孵化箱的容量校验。
 * 返回值：新生小鸡的列表槽位；孵化失败时为 -1。
 */
MaryChickenSlotIndex AttemptEggHatch(MaryChickenIncubatorIndex incubator_index);

/*
 * Tests whether a barn feed trough is already filled.
 * Parameter: trough_index uses BARN_FEED_TROUGH_* for ordinary troughs or
 * BARN_PREGNANCY_FEED_TROUGH_* for pregnancy-stall troughs. The handler checks
 * the selected group against the current barn-upgrade capacity.
 * Return value: nonzero when fodder is present; zero when the trough is empty.
 *
 * 判断牛羊小屋中的指定饲料槽是否已经放入饲料。
 * 参数：trough_index 使用 BARN_FEED_TROUGH_* 选择普通槽，或使用
 * BARN_PREGNANCY_FEED_TROUGH_* 选择怀孕槽；处理函数会按当前牛羊舍扩建
 * 状态校验对应分组的容量。
 * 返回值：已有牧草时为非零，饲料槽为空时为零。
 */
MaryBool IsBarnFeedTroughFilled(MaryBarnFeedTroughIndex trough_index);

/*
 * Fills a barn feed trough with fodder.
 * Parameter: trough_index uses the same normal-stall and pregnancy-stall
 * encoding as IsBarnFeedTroughFilled. Feed-box scripts normally check the
 * trough first; scripted tutorials may fill it directly.
 *
 * 向牛羊小屋中的指定饲料槽放入牧草。
 * 参数：trough_index 使用与 IsBarnFeedTroughFilled 相同的普通畜栏和怀孕畜栏
 * 编码。饲料箱脚本通常会先检查饲料槽；教学事件也可能直接放入牧草。
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
 *
 * 获取当前畜舍中可用的妊娠栏数量。
 * 参数：无。
 * 返回值：畜舍扩建前为 1，扩建后为 2。
 */
int GetPregnancyStallCapacity(void);

/*
 * Tests whether the cow or sheep in a pregnancy stall is ready to give birth.
 * Parameter: pregnancy_stall_index uses BARN_PREGNANCY_STALL_*; only the
 * first stall exists before the barn upgrade.
 * Return value: nonzero when birth is due; zero otherwise.
 *
 * 判断指定妊娠栏中的牛或羊是否已经到达生产日。
 * 参数：pregnancy_stall_index 使用 BARN_PREGNANCY_STALL_*；扩建前只有
 * 第一个怀孕槽存在。
 * 返回值：可以生产时为非零，否则为零。
 */
MaryBool IsBarnAnimalReadyToGiveBirth(MaryBarnPregnancyStallIndex pregnancy_stall_index);

/*
 * Delivers a ready cow or sheep and inserts the newborn into the barn roster.
 * Parameter: pregnancy_stall_index uses BARN_PREGNANCY_STALL_* and is checked
 * against the current one- or two-stall capacity.
 * Return value: the newborn's barn roster slot, or -1 if birth fails.
 *
 * 让已经到期的牛或羊生产，并将幼崽加入畜舍动物列表。
 * 参数：pregnancy_stall_index 使用 BARN_PREGNANCY_STALL_*，并按当前一个
 * 或两个怀孕槽的容量校验。
 * 返回值：新生动物的畜舍列表槽位；生产失败时为 -1。
 */
MaryAnimalSlotIndex AttemptBarnAnimalBirth(MaryBarnPregnancyStallIndex pregnancy_stall_index);

/*
 * Constructs and unlocks the mountain cottage awarded by the fiftieth wedding
 * anniversary event.
 * Parameters: none. The surrounding event sets its own
 * completion variable before invoking this persistent world-state change.
 *
 * 建造并解锁结婚五十周年事件奖励的山顶别墅。
 * 参数：无。外围事件会先设置自身的完成变量，再调用此函数修改持久世界状态。
 */
void BuildMountainCottage(void);

/*
 * Permanently builds the Seaside Cottage on Mineral Beach. The shipped
 * scripts call this after the Harvest Goddess announces the maximum link
 * level reward. This is the companion flag operation to BuildMountainCottage.
 *
 * 永久建成矿石海滩的海边别墅。原版脚本在女神宣布联动度满级奖励后调用本
 * 函数；它是与 BuildMountainCottage 对应的另一项别墅持久标志操作。
 */
void BuildSeasideCottage(void);

/*
 * Tests whether at least one piece of Golden Lumber is placed on the farm.
 * Parameters: none.
 * Return value: nonzero when a placed farm plot contains Golden Lumber; zero
 * otherwise. Daily dialogue and achievement scripts use this for the special
 * villager reaction to displaying Golden Lumber.
 *
 * 判断农场中是否至少摆放了一块黄金资材。
 * 参数：无。
 * 返回值：农场地块中摆放了黄金资材时为非零，否则为零。每日对话和成就脚本
 * 用它触发村民对展示黄金资材的特殊反应。
 */
MaryBool HasGoldenLumberOnFarm(void);

/*
 * Opens a map door or entrance by its runtime door ID. Scripts pair this with
 * the opening sound and movement through the entrance.
 * Parameter: door_index is the current map's local door/entrance index. It is
 * not a global location identifier.
 *
 * 按运行时门编号打开地图中的门或入口。脚本通常将其与开门音效及穿门移动配合。
 * 参数：door_index 为当前地图局部的门／入口索引，并非全局地点编号。
 */
void OpenDoor(MaryDoorIndex door_index);

/*
 * Closes a map door or entrance previously opened by OpenDoor.
 * Parameter: door_index uses the same current-map local domain as OpenDoor.
 *
 * 关闭此前由 OpenDoor 打开的地图门或入口。
 * 参数：door_index 与 OpenDoor 使用同一当前地图局部索引域。
 */
void CloseDoor(MaryDoorIndex door_index);

/*
 * Applies the next purchased rucksack capacity upgrade. The supermarket calls
 * this after charging 3000G or 5000G for the corresponding rucksack tier.
 * Parameters: none.
 *
 * 应用下一档已购买的背包容量升级。杂货店在为相应背包档位扣除 3000G 或
 * 5000G 后调用此函数。
 * 参数：无。
 */
void CompleteRucksackUpgrade(void);

/*
 * Completes the supermarket Blue Feather purchase. Unlike ordinary inventory
 * insertion, this callable performs the purchase-specific state update after
 * the Blue Feather has been placed in the player's tool inventory.
 * Parameters: none.
 *
 * 完成杂货店蓝色羽毛购买流程。与普通的背包写入不同，本函数在蓝色羽毛已经
 * 放入玩家工具栏后更新该商品专用的购买状态。
 * 参数：无。
 */
void CompleteBlueFeatherPurchase(void);

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
int GetHarvestSpriteWorkDaysLeft(MaryCharacterId sprite_id);

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
int GetHarvestSpriteTaskExperience(MaryCharacterId sprite_id, MaryHarvestSpriteTask task);

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
 * task category. Parameters use the same domains as
 * GetHarvestSpriteTaskExperience(). Return value: nonzero when experience is
 * present for that task; zero otherwise.
 *
 * 判断指定小矮人在某类工作小游戏中是否拥有非零经验。参数范围与
 * GetHarvestSpriteTaskExperience() 相同。
 * 返回值：该工作已有经验时为非零，
 * 否则为零。
 */
MaryBool HasHarvestSpriteTaskExperience(
    MaryCharacterId sprite_id,
    MaryHarvestSpriteTask task
);

/*
 * Assigns work to a Harvest Sprite.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID; task is a working
 * HARVEST_SPRITE_TASK_* value; days is the assignment duration in days.
 * Hiring dialogue commonly passes 1, 3, or 7 days.
 *
 * 为指定小矮人安排工作。
 * 参数：sprite_id 为该小矮人的 CHARACTER_* ID；task 为实际工作用的
 * HARVEST_SPRITE_TASK_*；days 为委托天数。雇佣对话通常传入 1、3 或 7 天。
 */
void StartHarvestSpriteTask(MaryCharacterId sprite_id, MaryHarvestSpriteTask task, int days);

/*
 * Stops the selected Harvest Sprite's current work for the rest of the day.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 *
 * 让指定小矮人停止当天剩余时间的当前工作。参数 sprite_id 为该小矮人的
 * CHARACTER_* ID。
 */
void StopHarvestSpriteWorkForToday(MaryCharacterId sprite_id);

/*
 * Returns 1 when the selected Harvest Sprite has completed or exhausted the
 * work currently available for its assigned task, otherwise 0.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 *
 * 当指定小矮人已完成当前任务可执行的工作，或当天已无可继续处理的目标时返回
 * 1，否则返回 0。参数 sprite_id 为该小矮人的 CHARACTER_* ID。
 */
MaryBool IsHarvestSpriteWorkComplete(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's animal-care training minigame and waits
 * for it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 *
 * 运行指定小矮人的动物照料训练小游戏，并等待小游戏结束。成功或能力提升结果
 * 返回 1，未成功结果返回 0。参数 sprite_id 为该小矮人的 CHARACTER_* ID。
 */
MaryBool RunHarvestSpriteAnimalCareMinigame(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's harvesting training minigame and waits
 * for it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 *
 * 运行指定小矮人的收获训练小游戏，并等待小游戏结束。成功或能力提升结果返回
 * 1，未成功结果返回 0。参数 sprite_id 为该小矮人的 CHARACTER_* ID。
 */
MaryBool RunHarvestSpriteHarvestingMinigame(MaryCharacterId sprite_id);

/*
 * Runs the selected Harvest Sprite's watering training minigame and waits for
 * it to finish. Returns 1 for the successful/improved result and 0 for the
 * unsuccessful result.
 * Parameters: sprite_id is the sprite's CHARACTER_* ID.
 *
 * 运行指定小矮人的浇水训练小游戏，并等待小游戏结束。成功或能力提升结果返回
 * 1，未成功结果返回 0。参数 sprite_id 为该小矮人的 CHARACTER_* ID。
 */
MaryBool RunHarvestSpriteWateringMinigame(MaryCharacterId sprite_id);

/*
 * Runs the Chicken Festival contest sequence and waits for its result. Returns
 * 1 when the player's chicken wins and 0 otherwise.
 * Parameters: none.
 *
 * 运行斗鸡节比赛流程并等待比赛结果。玩家的鸡获胜时返回 1，否则返回 0。
 * 无参数。
 */
MaryFestivalContestResult RunChickenFestivalContest(void);

/*
 * Runs the horse-race interface.
 * Parameter: race_mode is HORSE_RACE_MODE_*.
 * Return value: -1 when the interface is cancelled, 0 when it exits without a
 * race result, 1 when the player horse wins, or 2 when it loses.
 *
 * 运行赛马界面。参数 race_mode 为 HORSE_RACE_MODE_*。
 * 返回值：取消界面时为
 * -1；未产生比赛结果而退出时为 0；玩家的马获胜时为 1，落败时为 2。
 */
MaryHorseRaceInterfaceResult RunHorseRace(MaryHorseRaceMode race_mode);

/*
 * Rebuilds the horse-race entry records, including the randomized NPC horse
 * identifiers and race attributes.
 * Parameter: entry_mode is
 * HORSE_RACE_ENTRIES_*; INCLUDE_PLAYER_HORSE marks the player's horse as an
 * entrant, while NPC_ONLY prepares a field without it. The invitation scripts
 * select INCLUDE_PLAYER_HORSE only after the player accepts participation.
 *
 * 重新生成赛马参赛记录，包括随机化的 NPC 马匹编号及比赛属性。
 * 参数 entry_mode 为 HORSE_RACE_ENTRIES_*；INCLUDE_PLAYER_HORSE 将玩家的马
 * 标记为参赛，NPC_ONLY 则生成不含玩家马匹的阵容。邀请事件只在玩家同意参赛后
 * 使用 INCLUDE_PLAYER_HORSE。
 */
void PrepareHorseRaceEntries(MaryHorseRaceEntryMode entry_mode);

/*
 * Opens the horse-race medal exchange interface and waits until it closes.
 *
 * 打开赛马奖牌兑换界面并等待其关闭。
 */
void OpenHorseRaceMedalExchange(void);

/*
 * Runs the dog-frisbee interface.
 * Parameter: game_mode is FRISBEE_MODE_*.
 * Return value is FESTIVAL_CONTEST_RESULT_*; contest scripts store it directly
 * in VAR_FRISBEE_TOURNAMENT_RESULT, while practice scripts may ignore it.
 *
 * 运行爱犬飞盘界面。参数 game_mode 为 FRISBEE_MODE_*。返回值为引擎的飞盘
 * 结果为 FESTIVAL_CONTEST_RESULT_*；比赛脚本会将其直接写入
 * VAR_FRISBEE_TOURNAMENT_RESULT，自由练习脚本可以忽略该值。
 */
MaryFestivalContestResult RunFrisbeeGame(MaryFrisbeeMode game_mode);

/* Runs the Frisbee Tournament round entered from the beach-rules sign while
 * the tournament state is active. Normal-day practice uses
 * RunFrisbeeGame(FRISBEE_MODE_PRACTICE) instead. The tournament path creates
 * its own modal task in every ROM (MFoMT-US kind 0x28; JP kind 0x27).
 *
 * 在飞盘大会状态生效时，从海滩规则牌入口运行大会回合。普通日期的练习改走
 * RunFrisbeeGame(FRISBEE_MODE_PRACTICE)。四个 ROM 的大会路径都会创建独立的
 * 模态任务（MFoMT-US 类型 0x28，JP 类型 0x27）。 */
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
 *
 * 检查等待中的电视购物订单是否已经可以送达。
 * 参数：无。
 * 返回值：存在订单且送货倒计时已经归零时为 1，否则为 0。
 */
MaryBool IsTVShoppingDeliveryReady(void);

/*
 * Selects the item currently offered by the TV Shopping program.
 * Parameter: item_id is TV_SHOPPING_ITEM_* or the exact original item ID.
 * This changes the current selection but does not yet place an order.
 *
 * 选择电视购物节目当前展示的商品。
 * 参数：item_id 为 TV_SHOPPING_ITEM_* 或精确原始商品 ID。
 * 本函数只改变当前选择，尚不会正式下单。
 */
void SetTVShoppingSelection(MaryTVShoppingItemId item_id);

/*
 * Confirms the current TV Shopping selection as the pending delivery order.
 * Parameters: none. Call SetTVShoppingSelection first when changing the item.
 *
 * 将当前电视购物选择确认为等待送货的订单。
 * 参数：无。需要更换商品时，应先调用 SetTVShoppingSelection。
 */
void ConfirmTVShoppingOrder(void);

/*
 * Completes the pending TV Shopping delivery and clears the stored order.
 * Parameters: none. For farmhouse items, the matching facility or utensil is
 * installed; the Power Berry delivery is finalized after its script-side effect.
 *
 * 完成等待中的电视购物送货并清除已保存订单。
 * 参数：无。农舍商品会安装对应设施或厨具；力量果实则在脚本侧效果完成后，
 * 由本函数结束该笔送货。
 */
void CompleteTVShoppingDelivery(void);

/*
 * Tests whether the Vacation Villa has been built.
 * Parameters: none.
 * Return value: nonzero after Gotz's Vacation Villa construction completes;
 * zero otherwise.
 *
 * 判断别墅（Vacation Villa）是否已经建成。
 * 参数：无。
 * 返回值：Gotz 完成别墅施工后为非零，否则为零。
 */
MaryBool IsVacationVillaBuilt(void);

/*
 * Tests the all-villagers maximum-friendship completion condition.
 * Parameters: none.
 * Return value: nonzero when the condition is satisfied; zero otherwise.
 *
 * 判断是否满足全村民最高友好度完成条件。
 * 参数：无。
 * 返回值：满足条件时为非零，否则为零。
 */
MaryBool AreAllVillagersAtMaxFriendship(void);

/*
 * Tests whether at least one of every crop has been shipped.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 判断是否每种农作物都至少出货过一个。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasShippedOneOfEachCrop(void);

/*
 * Tests the all-farm-animals maximum-affection completion condition.
 * Parameters: none.
 * Return value: nonzero when the condition is satisfied; zero otherwise.
 *
 * 判断是否满足所有农场动物最高好感度完成条件。
 * 参数：无。
 * 返回值：满足条件时为非零，否则为零。
 */
MaryBool AreAllFarmAnimalsAtMaxAffection(void);

/*
 * Tests whether at least one of every mineral has been shipped.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 判断是否每种矿石都至少出货过一个。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasShippedOneOfEachMineral(void);

/*
 * Tests whether every fish species has been caught.
 * Parameters: none.
 * Return value: nonzero when complete; zero otherwise.
 *
 * 判断是否已捕获全部鱼类。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool HasCaughtEveryFishSpecies(void);

/*
 * Returns the cumulative number of fish caught.
 * Parameters: none.
 * Return value: total fish caught across the save data.
 *
 * 返回累计捕获鱼的数量。
 * 参数：无。
 * 返回值：存档中的累计捕获数量。
 */
int GetTotalFishCaught(void);

/*
 * Tests whether the player has obtained a Mythic Tool.
 * Parameters: none.
 * Return value: nonzero after any Mythic-level sickle, hoe, axe, hammer,
 * watering can, or fishing rod is first awarded; zero otherwise.
 *
 * 判断玩家是否曾获得 Mythic（贤者）农具。
 * 参数：无。
 * 返回值：首次取得任一 Mythic 等级的镰刀、锄头、斧、锤、洒水壶或钓竿后
 * 为非零，否则为零。
 */
MaryBool HasObtainedMythicTool(void);

/* Tests the broader shipping completion flag covering every product category,
 * rather than only crops or minerals. Return value: nonzero after at least
 * one of every required product has been shipped; zero otherwise.
 *
 * 判断覆盖全部出货品类别的总完成状态，而不是仅检查作物或矿物。
 * 返回值：所有要求的出货品都至少出货一次后为非零，否则为零。
 */
MaryBool HasShippedOneOfEachProduct(void);

/*
 * Returns the player's current money.
 * Parameters: none.
 * Return value: current game-currency amount.
 *
 * 返回玩家当前金钱。
 * 参数：无。
 * 返回值：当前游戏货币数量。
 */
int GetMoney(void);

/*
 * Adds game currency to the player.
 * Parameter: amount is the currency amount to add.
 *
 * 增加玩家的游戏货币。
 * 参数：amount 为增加的货币数量。
 */
void AddMoney(int amount);

/*
 * Subtracts game currency from the player.
 * Parameter: amount is the currency amount to subtract.
 *
 * 扣除玩家的游戏货币。
 * 参数：amount 为扣除的货币数量。
 */
void SubtractMoney(int amount);

/*
 * Enables the global scripted-NPC-control mode used while a coordinated event
 * owns participant movement and per-NPC event scripts. In this mode the normal
 * NPC schedule/update path observes the event-control flag instead of freely
 * advancing participants. Pair it with DisableScriptedNpcControl when the
 * event's participant scripts have finished.
 * Parameters: none.
 * Return value: none.
 *
 * 启用全局的脚本化 NPC 控制模式，用于由事件统一控制参与者移动和各 NPC
 * 事件脚本的场景。启用后，常规 NPC 日程/更新流程会遵循事件控制标志，
 * 不再自行推进参与者。参与者脚本结束后应调用 DisableScriptedNpcControl。
 * 参数：无。
 * 返回值：无。
 */
void EnableScriptedNpcControl(void);

/*
 * Disables scripted-NPC-control mode and returns NPC updates to the normal
 * schedule path. Event cleanup scripts call this before removing or restoring
 * their participant entities.
 * Parameters: none.
 * Return value: none.
 *
 * 关闭脚本化 NPC 控制模式，使 NPC 更新恢复到常规日程流程。事件清理脚本会在
 * 移除或恢复参与者实体之前调用它。
 * 参数：无。
 * 返回值：无。
 */
void DisableScriptedNpcControl(void);

/*
 * Returns the current blacksmith order ID.
 * Parameters: none.
 * Return value: a BLACKSMITH_ORDER_* value; BLACKSMITH_ORDER_NONE means no
 * order is pending.
 *
 * 返回当前锻冶屋订单 ID。
 * 参数：无。
 * 返回值：BLACKSMITH_ORDER_*；BLACKSMITH_ORDER_NONE 表示当前没有订单。
 */
MaryBlacksmithOrderId GetBlacksmithOrderId(void);

/*
 * Tests whether the current blacksmith order is ready.
 * Parameters: none.
 * Return value: nonzero when ready; zero otherwise.
 *
 * 判断当前锻冶屋订单是否完成。
 * 参数：无。
 * 返回值：完成时为非零，否则为零。
 */
MaryBool IsBlacksmithOrderReady(void);

/*
 * Tries to collect the completed blacksmith order. This operation places the
 * result in the player's held slot, rucksack, or appropriate storage and
 * clears the pending order when placement succeeds.
 * Parameters: none. Call only after IsBlacksmithOrderReady() succeeds.
 * Return value: a BLACKSMITH_COLLECTION_* placement result. NO_SPACE leaves
 * the order pending so collection can be retried.
 *
 * 尝试领取已经完成的锻冶屋订单。该操作会把成品放入玩家手持栏、背包或对应
 * 仓库；成功放入后会清除等待中的订单。
 * 参数：无。应在 IsBlacksmithOrderReady() 成功后调用。
 * 返回值：BLACKSMITH_COLLECTION_* 放置结果。NO_SPACE 不会清除订单，可在
 * 腾出空间后再次领取。
 */
MaryBlacksmithCollectionResult CollectBlacksmithOrder(void);

/*
 * Sets the in-game clock immediately.
 * Parameters: hour is the 0-23 hour; minute is the 0-59 minute. The engine
 * stores them in its five-bit hour and six-bit minute fields and refreshes
 * dependent game state.
 *
 * 立即设置游戏内时钟。
 * 参数：hour 为 0-23 时，minute 为 0-59 分。引擎会分别写入 5 位小时字段和
 * 6 位分钟字段，并刷新依赖时间的游戏状态。
 */
void SetGameTime(int hour, int minute);

/*
 * Tests whether a letter is waiting for delivery or readout.
 * Parameter: letter_id is a REFERENCE_PAGE_* entry from the selected target's
 * physical page table, or the exact numeric ID.
 * Return value: nonzero when waiting; zero otherwise.
 *
 * 判断指定邮件是否处于待投递或待读取状态。
 * 参数：letter_id 为所选目标物理页面表中的 REFERENCE_PAGE_*，或精确数字 ID。
 * 返回值：处于待收状态时为非零，否则为零。
 */
MaryBool IsLetterWaiting(MaryReferencePageId letter_id);

/*
 * Tests whether a letter has been received persistently.
 * Parameter: letter_id is a REFERENCE_PAGE_* entry or the exact numeric ID.
 * Return value: nonzero when received; zero otherwise.
 *
 * 判断指定邮件是否已经永久收取。
 * 参数：letter_id 为 REFERENCE_PAGE_* 条目或精确数字 ID。
 * 返回值：已经收取时为非零，否则为零。
 */
MaryBool HasReceivedLetter(MaryReferencePageId letter_id);

/*
 * Queues or delivers a letter.
 * Parameter: letter_id is a REFERENCE_PAGE_* entry or the exact numeric ID.
 * Use IsLetterWaiting and HasReceivedLetter to inspect its lifecycle state.
 *
 * 投递或加入指定邮件。
 * 参数：letter_id 为 REFERENCE_PAGE_* 条目或精确数字 ID。
 * 可使用 IsLetterWaiting 与 HasReceivedLetter 查询其生命周期状态。
 */
void DeliverLetter(MaryReferencePageId letter_id);

/*
 * Marks a letter as read.
 * Parameter: letter_id is a REFERENCE_PAGE_* entry or the exact numeric ID.
 *
 * 将指定邮件标记为已读。
 * 参数：letter_id 为 REFERENCE_PAGE_* 条目或精确数字 ID。
 */
void MarkLetterRead(MaryReferencePageId letter_id);

/*
 * Returns the number of letters waiting for delivery or readout.
 * Parameters: none.
 * Return value: waiting-letter count.
 *
 * 返回待投递或待读取邮件的数量。
 * 参数：无。
 * 返回值：待收邮件数量。
 */
int GetWaitingLetterCount(void);

/*
 * Returns the number of saved or persistently received letters.
 * Parameters: none.
 * Return value: saved-letter count.
 *
 * 返回已保存或永久收取邮件的数量。
 * 参数：无。
 * 返回值：已保存邮件数量。
 */
int GetSavedLetterCount(void);

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
 *
 * 通过电视观看界面显示一段节目文本，并等待导航操作。它与 TalkMessage 不同，
 * 会返回供调用脚本切换频道或退出节目使用的观看界面操作值。
 * 参数：message 为当前脚本 mary_text_table 中的字符串。
 * 返回值：MaryTelevisionInput。方向键结果用于选择频道；
 * TELEVISION_INPUT_ADVANCE_TEXT 继续当前文本，TELEVISION_INPUT_TURN_OFF
 * 离开电视界面。
 * 需要特定节目画面时先调用 SetTelevisionProgram，最后一段文本后调用
 * EndTelevisionProgram。
 */
MaryTelevisionInput ShowTelevisionMessage(const char *message);

/*
 * Selects the television program presentation used by subsequent television
 * messages. The value is a MaryTelevisionProgramId, not a script ID.
 * Parameters: program_id is the television program/presentation slot.
 * Return value: none.
 *
 * 选择后续电视文本使用的节目显示样式。该值是 MaryTelevisionProgramId，
 * 不是脚本 ID。
 * 参数：program_id 为电视节目/显示槽位。
 * 返回值：无。
 */
void SetTelevisionProgram(MaryTelevisionProgramId program_id);

/*
 * Ends the active television-program presentation after its final message.
 * Parameters: none.
 * Return value: none.
 *
 * 在最后一段节目文本完成后结束当前电视节目显示。
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
 * Tests whether an animal kind/index pair resolves to a live animal slot.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: nonzero for a live slot; zero for an empty or invalid slot.
 * Call this before indexed animal getters while iterating barn or coop slots.
 *
 * 判断动物类别与槽位组合是否指向有效动物。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该动物类别内从 0 开始的槽位。
 * 返回值：有效动物槽为非零；空槽或无效槽为零。
 * 遍历牛羊棚或鸡舍槽位时，应先调用本函数再读取动物属性。
 */
MaryBool DoesAnimalExist(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Creates and registers the farm's horse, then creates its runtime entity.
 * Parameters: facing is the initial engine direction; age_stage is a
 * MaryHorseAgeStage whose value is converted to age_stage * 120 days;
 * map_id is the initial MaryMapId; x and y are absolute map coordinates.
 * The shipped horse-offer events use entity slot 44 internally after this
 * call.
 *
 * 创建并登记农场的马，随后创建其运行时实体。
 * 参数：facing 为初始引擎朝向；age_stage 为 MaryHorseAgeStage，引擎会将其
 * 换算为 age_stage * 120 天；map_id 为初始 MaryMapId；x、y 为绝对地图坐标。
 * 原版送马事件在本调用后使用运行时实体槽 44。
 */
void CreateFarmHorse(
    MaryFacingDirection facing,
    MaryHorseAgeStage age_stage,
    MaryMapId map_id,
    MaryMapSpaceX x,
    MaryMapSpaceY y);

/*
 * Removes runtime horse entity 44 and clears the registered farm horse when
 * remove_guard is zero. A nonzero remove_guard makes the native routine return
 * without changing either object; it is a guard value, not a family of removal
 * modes. The reserved argument is consumed by the VM wrapper but never passed
 * to the native routine. All shipped calls pass zero for both arguments. FoMT
 * source and the MFoMT-US/MFoMT-JP native handlers agree on this behavior.
 *
 * 当 remove_guard 为零时，删除运行时马实体 44，并清除农场数据中登记的马；
 * 非零值会让原生函数直接返回，不改变两处对象，所以它是移除保护值而不是多种
 * “移除模式”。reserved 会被 VM 包装层从栈中取出，但不会传给原生函数。原版
 * 脚本的两个参数均为零。FoMT 源码与 MFoMT-US/MFoMT-JP 原生处理函数对此一致。
 */
void RemoveFarmHorse(MaryBool remove_guard, int reserved);

/*
 * Copies the selected animal's name into a text string-substitution slot.
 * Parameters: text_variable is TEXT_VARIABLE_1 through TEXT_VARIABLE_4;
 * animal_kind is ANIMAL_KIND_*; animal_index is the zero-based slot in that
 * family.
 * Use DoesAnimalExist before calling this with a dynamically iterated index.
 *
 * 将指定动物名称复制到文本字符串替换槽。
 * 参数：text_variable 为 TEXT_VARIABLE_1 至 TEXT_VARIABLE_4；animal_kind 为
 * ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 动态遍历槽位时，应先使用 DoesAnimalExist 检查。
 */
void GetAnimalName(
    MaryTextVariableSlot text_variable,
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index
);

/*
 * Returns the zero-based livestock slot bound to the currently executing
 * animal-interaction event.
 * Parameters: none. The event determines the animal
 * family separately, then uses this slot with the typed animal getters and
 * setters. This is not a global entity ID.
 *
 * 返回当前正在执行的动物交互事件所绑定的家畜槽位（从 0 开始）。
 * 参数：无。事件会另外确定动物类别，再将此槽位传给带类型的动物属性读写函数。
 * 该返回值不是全局实体 ID。
 */
MaryAnimalSlotIndex GetInteractingAnimalIndex(void);

/*
 * Tests the selected animal's daily talked-to flag.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: nonzero when already talked to today; zero otherwise.
 *
 * 判断指定动物当日是否已经交谈。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：当日已经交谈时为非零，否则为零。
 */
MaryBool HasAnimalBeenTalkedTo(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Sets the selected animal's daily talked-to flag.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Pair this with HasAnimalBeenTalkedTo around the completed interaction.
 *
 * 设置指定动物的当日已交谈标志。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 应与 HasAnimalBeenTalkedTo 配合，在交互完成时设置。
 */
void SetAnimalTalkedTo(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Adds a signed amount to the selected animal's affection.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family; amount is the signed affection change.
 *
 * 为指定动物增加有符号好感度变化量。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * amount 为有符号好感度变化量。
 */
void AddAnimalAffection(
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index,
    int amount
);

/*
 * Tests whether the selected livestock animal is unhappy.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: nonzero when unhappy; zero otherwise.
 *
 * 判断指定家畜是否处于不高兴状态。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：不高兴时为非零，否则为零。
 */
MaryBool IsAnimalUnhappy(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected livestock animal is sick.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: nonzero when sick; zero otherwise.
 *
 * 判断指定家畜是否生病。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：生病时为非零，否则为零。
 */
MaryBool IsAnimalSick(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected barn animal is pregnant.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: nonzero when pregnant; zero otherwise.
 *
 * 判断指定牛羊棚动物是否怀孕。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：怀孕时为非零，否则为零。
 */
MaryBool IsAnimalPregnant(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Returns the healthy-pregnancy day counter for the selected barn animal.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: the engine's healthy-pregnancy day value.
 *
 * 返回指定牛羊棚动物的健康怀孕天数计数。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：引擎记录的健康怀孕天数值。
 */
int GetAnimalHealthyPregnancyDays(
    MaryAnimalKind animal_kind,
    MaryAnimalSlotIndex animal_index
);

/* Returns the selected animal's age counter.
 * Parameters use the same animal family and zero-based slot convention as the
 * neighboring animal-state queries.
 *
 * 返回指定动物的年龄计数。参数使用与相邻动物状态查询相同的动物类别和
 * 从 0 开始的槽位约定。
 */
int GetAnimalAge(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Returns the selected animal's affection value.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: current animal affection.
 *
 * 返回指定动物的好感度。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：动物当前好感度。
 */
int GetAnimalAffection(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Returns the selected animal's growth-stage value.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the zero-based
 * slot within that animal family.
 * Return value: the engine growth-stage value for horse, cow, sheep, chicken,
 * or dog as selected by animal_kind. When animal_kind is a statically known
 * symbol, the decompiler uses MaryPetGrowthStage, MaryCowGrowthStage,
 * MarySheepGrowthStage, or MaryChickenGrowthStage as appropriate.
 *
 * 返回指定动物的成长阶段值。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该类别内从 0 开始的槽位。
 * 返回值：由 animal_kind 选择的马、牛、羊、鸡或狗的引擎成长阶段值。当
 * animal_kind 是静态可知的符号时，反编译器会按物种选用 MaryPetGrowthStage、
 * MaryCowGrowthStage、MarySheepGrowthStage 或 MaryChickenGrowthStage。
 */
int GetAnimalGrowthStage(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Tests whether the selected sheep has been sheared.
 * Parameter: sheep_index is the zero-based sheep slot in the shared barn.
 * Return value: nonzero when sheared; zero otherwise.
 *
 * 判断指定羊是否已经剪毛。
 * 参数：sheep_index 为共享牛羊棚中从 0 开始的羊槽位。
 * 返回值：已经剪毛时为非零，否则为零。
 */
MaryBool IsSheepSheared(MaryAnimalSlotIndex sheep_index);

/*
 * Counts cows, sheep, or chickens whose stored life state matches the requested
 * value.
 * Parameters: animal_kind is ANIMAL_KIND_COW, ANIMAL_KIND_SHEEP, or
 * ANIMAL_KIND_CHICKEN; life_state is LIVESTOCK_LIFE_STATE_*. Return value: the
 * number of matching livestock slots. The death-summary event uses the two
 * death states before its cleanup removes those records.
 *
 * 统计保存的生命状态等于指定值的牛、羊或鸡。
 * 参数：animal_kind 为 ANIMAL_KIND_COW、ANIMAL_KIND_SHEEP 或
 * ANIMAL_KIND_CHICKEN；life_state 为 LIVESTOCK_LIFE_STATE_*。
 * 返回值：匹配的家畜槽位数量。动物死亡汇总事件会在清理相应记录前读取两种死亡状态。
 */
int CountAnimalsByLifeState(
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
 * Return value: nonzero for a cow; zero otherwise.
 *
 * 判断共享牛羊棚槽位中是否为牛。
 * 参数：barn_slot 为从 0 开始的共享牛羊棚槽位。
 * 返回值：槽位中为牛时为非零，否则为零。
 */
MaryBool IsCowAtBarnSlot(MaryAnimalSlotIndex barn_slot);

/*
 * Returns the number of currently owned cows.
 * Parameters: none.
 * Return value: cow count.
 *
 * 返回当前拥有的牛数量。
 * 参数：无。
 * 返回值：牛数量。
 */
int GetCowCount(void);

/*
 * Returns the number of currently owned sheep.
 * Parameters: none.
 * Return value: sheep count.
 *
 * 返回当前拥有的羊数量。
 * 参数：无。
 * 返回值：羊数量。
 */
int GetSheepCount(void);

/*
 * Returns the number of currently owned chickens.
 * Parameters: none.
 * Return value: chicken count.
 *
 * 返回当前拥有的鸡数量。
 * 参数：无。
 * 返回值：鸡数量。
 */
int GetChickenCount(void);

/*
 * Registers one owned animal as the entrant for its contest family.
 * Parameters: animal_kind is ANIMAL_KIND_*; animal_index is the animal's
 * zero-based slot within that family.
 * Related calls: GetContestAnimalIndex reads the registered slot, and
 * ClearContestAnimal removes the registration after the contest or event.
 *
 * 将一只已拥有的动物登记为相应类别比赛的参赛动物。
 * 参数：animal_kind 为 ANIMAL_KIND_*；animal_index 为该动物在类别内从 0 开始的槽位。
 * 联动调用：GetContestAnimalIndex 读取登记槽位；比赛或事件结束后由
 * ClearContestAnimal 清除登记。
 */
void SetContestAnimal(MaryAnimalKind animal_kind, MaryAnimalSlotIndex animal_index);

/*
 * Clears the registered contest entrant for one animal family.
 * Parameter: animal_kind is ANIMAL_KIND_*.
 * Related calls: SetContestAnimal registers the entrant and
 * GetContestAnimalIndex reads its slot.
 *
 * 清除一个动物类别中已经登记的比赛参赛动物。
 * 参数：animal_kind 为 ANIMAL_KIND_*。
 * 联动调用：SetContestAnimal 负责登记；GetContestAnimalIndex 读取其槽位。
 */
void ClearContestAnimal(MaryAnimalKind animal_kind);

/*
 * Returns the selected contest entrant slot for an animal family.
 * Parameter: animal_kind is ANIMAL_KIND_*.
 * Return value: the selected animal's zero-based family slot.
 *
 * 返回指定动物类别所选择的比赛参赛槽位。
 * 参数：animal_kind 为 ANIMAL_KIND_*。
 * 返回值：所选动物在该类别内从 0 开始的槽位。
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
 * Return value: current catch size in the engine's native unit.
 *
 * 返回当前事件正在处理的捕获鱼尺寸。
 * 参数：无。
 * 返回值：使用引擎原生单位的当前捕获尺寸。
 */
int GetCaughtFishSize(void);

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
 * Tests whether the current catch is a River King.
 * Parameters: none.
 * Return value: nonzero for a River King; zero otherwise.
 *
 * 判断当前捕获是否为鱼王。
 * 参数：无。
 * 返回值：属于鱼王时为非零，否则为零。
 */
MaryBool IsCaughtFishRiverKing(void);

/*
 * Selects the partner index used by the upcoming Moon-Viewing Festival.
 * Parameters: none. Return value: MOON_VIEWING_PARTNER_*. A married player
 * gets the spouse; otherwise the engine considers candidates whose rival-event
 * progression still permits the event, selects the highest love above the
 * required threshold, and breaks equal scores randomly.
 *
 * 选择即将举行的赏月节所使用的同行对象索引。
 * 参数：无。
 * 返回值：MOON_VIEWING_PARTNER_*。玩家已婚时返回配偶；未婚时引擎
 * 仅考虑情敌事件进度仍允许参加的候选人，从爱情度达到门槛者中选择最高值，
 * 同分时随机决定。
 */
MaryMoonViewingPartner SelectMoonViewingPartner(void);
#if defined(MARY_MFOMT)
/*
 * Duplicate callable-table alias of SelectMoonViewingPartner. MFoMT-US and
 * MFoMT-JP dispatch slots 0x122 and 0x123 to the exact same native handler;
 * neither ROM adds a wrapper or changes its return convention. Prefer
 * SelectMoonViewingPartner in new source. This alias exists so raw bytecode
 * containing the second ID can be represented without losing that ID.
 *
 * SelectMoonViewingPartner 的 callable 表重复别名。MFoMT-US 与 MFoMT-JP 的
 * 0x122、0x123 两个槽都直接指向完全相同的原生处理器，不存在额外包装，也没有
 * 改变返回约定。新脚本应优先使用 SelectMoonViewingPartner；保留该别名是为了
 * 让含第二个 ID 的原始字节码能够表达并保持原 ID，不发生信息丢失。
 */
MaryMoonViewingPartner SelectMoonViewingPartnerAlias(void);
#endif
/*
 * Rates the food currently held by the player for the active Cooking Festival
 * theme.
 * Parameters: none. Return value: COOKING_FESTIVAL_DISH_RATING_*.
 * Ratings excellent and great win in the vanilla result script; ineligible
 * means the held food is invalid or does not match the year's category.
 *
 * 按当前料理祭主题评价玩家手中持有的料理。
 * 参数：无。
 * 返回值：COOKING_FESTIVAL_DISH_RATING_*。
 * 原版结果脚本中“优秀”和“很棒”会获胜；“不合格”表示手持料理无效或不符合当年类别。
 */
MaryCookingFestivalDishRating GetCookingFestivalDishRating(void);

/*
 * Selects the weighted mineral gift Thomas leaves in the stocking on Winter
 * 25.
 * Parameters: none. Return value: MaryThomasStockingGift values 1-5; the
 * delivery script maps them to Mystrile, Orichalc, Moon Stone, Sand Rose, or
 * Alexandrite respectively.
 *
 * 为冬 25 日袜子礼物事件按权重选择 Thomas 留下的矿石礼物。
 * 参数：无。
 * 返回值：MaryThomasStockingGift 的 1-5；送礼脚本依次把它们映射为
 * 秘银、奥利哈钢、月亮石、沙漠玫瑰石和亚历山大石。
 */
MaryThomasStockingGift SelectThomasStockingGift(void);

/*
 * Draws one article ID from the weighted gift table used by post-marriage
 * spouse-present events.
 * Parameters: none. Return value: the selected
 * MaryArticleId, suitable for SetPlayerHeldArticle or its wrapped variant.
 *
 * 从婚后配偶赠礼事件使用的加权礼物表中抽取一个物品 ID。
 * 参数：无。
 * 返回值：选中的 MaryArticleId，可直接传给 SetPlayerHeldArticle
 * 或其包装物品版本。
 */
MaryArticleId GetRandomSpouseGiftArticleId(void);

/*
 * Unlocks the next Van-shop music album made available by GameCube link
 * progress.
 * Parameters: none. Return value: 1 when the unlocked-album count
 * was increased, or 0 when all ten link albums were already unlocked. The
 * engine table maps these ten entries directly to ARTICLE_ALBUM_1 through
 * ARTICLE_ALBUM_10.
 *
 * 根据 GameCube 联动进度，解锁 Van 商店中下一张可出售的音乐唱片。
 * 参数：无。
 * 返回值：成功增加已解锁唱片数量时为 1；十张联动唱片均已解锁时为 0。
 * 引擎表会将这十项直接映射到 ARTICLE_ALBUM_1 至 ARTICLE_ALBUM_10。
 */
MaryBool UnlockNextVanAlbum(void);

/*
 * Tests whether all ten GameCube-link albums are currently available in
 * Van's shop.
 * Parameters: none. Return value: 1 only when every album slot
 * from Album 1 through Album 10 is unlocked and has not been removed from the
 * current shop inventory; otherwise 0.
 *
 * 检查十张 GameCube 联动唱片当前是否全部可在 Van 商店购买。
 * 参数：无。
 * 返回值：仅当 Album 1 至 Album 10 的全部槽位均已解锁，且尚未从当前
 * 商店库存中移除时为 1；否则为 0。
 */
MaryBool AreAllVanAlbumsAvailable(void);

/*
 * Stores a preset nickname that the player's spouse will use for the player.
 * Parameter: nickname points to a script text entry containing the selected
 * name. For a custom typed nickname, use OpenNameEntry with
 * NAME_ENTRY_SPOUSE_NICKNAME instead.
 *
 * 保存配偶称呼玩家时所使用的预设昵称。
 * 参数：nickname 指向包含所选称呼的脚本文本。需要由玩家输入自定义昵称时，
 * 应改用 OpenNameEntry(NAME_ENTRY_SPOUSE_NICKNAME, ...) 。
 */
void SetPlayerNicknameForSpouse(const char *nickname);

/*
 * Places the player at the farmhouse bed used after overnight or
 * event-ending transitions. The engine selects the bed X coordinate from the
 * current farmhouse-upgrade level and uses the fixed farmhouse-interior map
 * and bed Y coordinate.
 * Parameters: none.
 *
 * 将玩家放到过夜或事件结束转场使用的农舍床位。引擎根据当前农舍升级等级选择
 * 床位 X 坐标，并使用固定的农舍室内地图及床位 Y 坐标。
 * 参数：无。
 */
void PlacePlayerAtFarmhouseBed(void);

/*
 * Starts the fixed screen-color flash effect using GBA 5-bit RGB channels.
 * Parameters: red, green, and blue are each in the range 0..31. The engine
 * packs them as red | (green << 5) | (blue << 10) before updating the palette.
 * Fireworks events use the three full-intensity primary colors in sequence.
 *
 * 使用 GBA 的 5 位 RGB 通道启动固定的屏幕闪色效果。
 * 参数：red、green、blue 的范围均为 0..31。引擎按
 * red | (green << 5) | (blue << 10) 打包后更新调色板；烟火事件会依次使用
 * 三种满强度原色。
 */
void FlashScreenColor(int red, int green, int blue);

/*
 * Rebuilds the current map's entity manager at the start of the new-day
 * script. This reloads map actors for the new calendar state and applies a
 * preserved overnight return position when one is active.
 * Parameters: none.
 *
 * 在新一天脚本开始时重建当前地图的实体管理器。它会按照新的日期状态重新载入
 * 地图角色，并在存在已保存的过夜返回位置时恢复该位置。
 * 参数：无。
 */
void RebuildMapEntitiesForNewDay(void);

/*
 * Cures every currently sick livestock entity in the barn and coop.
 * Parameters: none. The shooting-star event uses this for the healthy-animals
 * wish; healthy livestock is left unchanged.
 *
 * 治愈牛羊棚与鸡舍中当前所有生病的家畜。
 * 参数：无。射星事件的“动物健康”愿望调用此函数；健康家畜不会发生变化。
 */
void CureAllSickLivestock(void);

/*
 * Adds the same signed affection amount to every owned farm animal: dog,
 * horse, chickens, cows, and sheep.
 * Parameter: amount is the signed affection
 * delta. Church-confession outcomes currently call this with a positive value.
 *
 * 为所有已拥有的农场动物增加相同的有符号好感变化量，包括狗、马、鸡、牛和羊。
 * 参数：amount 为有符号好感变化量；当前教堂忏悔结果会传入正值。
 */
void AddAffectionToAllFarmAnimals(int amount);

/*
 * Enables the persistent shipping-price bonus awarded by the shooting-star
 * event's shipping wish.
 * Parameters: none. This function sets the dedicated
 * event bonus flag; it does not immediately ship an item.
 *
 * 启用射星事件“提高出货价格”愿望所奖励的持久出货加成。
 * 参数：无。该函数设置专用事件奖励标志，不会立即出货任何物品。
 */
void EnableShootingStarShippingBonus(void);

/*
 * Returns the cumulative shipped count for a product.
 * Parameter: product_id is the target-specific MaryProductId. This is the
 * ShippingBin table domain, not MaryFoodId or MaryArticleId.
 * Return value: cumulative shipped amount for that entry.
 *
 * 返回指定产品的累计出货量。
 * 参数：product_id 为目标版本的 MaryProductId；它属于 ShippingBin 表编号域，
 * 不是 MaryFoodId 或 MaryArticleId。
 * 返回值：该表项的累计出货数量。
 */
int GetAmountShipped(MaryProductId product_id);

/*
 * Initializes the player's child record when needed, then creates and enables
 * the child actor entity (entity ID 35) for the current scene.
 * Parameters:
 * none. Childbirth scripts call this immediately before positioning the baby;
 * existing child data is preserved rather than initialized again.
 *
 * 在需要时初始化玩家孩子的资料，随后为当前场景创建并启用孩子角色实体
 *（实体 ID 35）。
 * 参数：无。生育事件会在设置婴儿位置前调用此函数；已经存在
 * 的孩子资料不会被重复初始化。
 */
void CreatePlayerChildEntity(void);

/*
 * Relocates a persistent actor entity to another map and stores its target
 * coordinates.
 * Parameters: entity_id is the runtime actor ID; map_id is a
 * MaryMapId; x and y are target map coordinates. Unlike SetEntityPosition,
 * this updates the actor's cross-map location and refreshes its entity state.
 *
 * 将持久角色实体迁移到另一张地图，并保存其目标坐标。
 * 参数：entity_id 为运行时
 * 角色 ID；map_id 为 MaryMapId；x、y 为目标地图坐标。它与
 * SetEntityPosition 不同，会更新角色的跨地图位置并刷新实体状态。
 */
void RelocateEntityToMap(
    MaryEntityId entity_id,
    MaryMapId map_id,
    MaryMapSpaceX x,
    MaryMapSpaceY y);

/*
 * Creates the dedicated first-sunrise visual entity used by the New Year
 * mountaintop event.
 * Parameters: none. Call PlayNewYearSunriseEffect next and
 * release the entity with DestroyNewYearSunriseEffect afterward.
 *
 * 创建跨年山顶事件专用的“新年首次日出”视觉实体。
 * 参数：无。随后应调用
 * PlayNewYearSunriseEffect，结束后再用 DestroyNewYearSunriseEffect 释放实体。
 */
void CreateNewYearSunriseEffect(void);

/*
 * Starts the previously created first-sunrise animation and waits until its
 * active transition finishes.
 * Parameters: none.
 *
 * 启动此前创建的新年首次日出动画，并等待其活动转场结束。
 * 参数：无。
 */
void PlayNewYearSunriseEffect(void);

/*
 * Destroys the first-sunrise visual entity after the New Year transition.
 * Parameters: none.
 *
 * 在跨年转场结束后销毁首次日出视觉实体。
 * 参数：无。
 */
void DestroyNewYearSunriseEffect(void);

/*
 * Plays the reusable star-sparkle particle effect and waits for its active
 * phase. Scripts use it both for the shooting-star night and for the Gourmet's
 * highest cooking-festival reaction.
 * Parameters: none.
 *
 * 播放可复用的星光闪烁粒子特效，并等待其活动阶段结束。脚本会在流星雨之夜及
 * 美食家对料理祭最高评价时使用它。
 * 参数：无。
 */
void PlayStarSparkleEffect(void);

/*
 * Counts recipe records whose learned flag is set. The cooking menu uses a
 * nonzero result to decide whether the recipe-list screen can be opened.
 * Parameters: none.
 * Return value: the number of known recipes.
 *
 * 统计已设置“学会”标志的料理记录。料理菜单以结果是否非零决定能否打开菜谱
 * 列表画面。
 * 参数：无。
 * 返回值：已经掌握的菜谱数量。
 */
int GetKnownRecipeCount(void);

/*
 * Generates the procedural 28-by-28 layout for one mine floor while excluding
 * tiles occupied by the floor's fixed entrance and exit entities.
 * Parameters:
 * mine_kind selects MINE_SPRING or MINE_LAKE; floor_index is zero-based within
 * that mine's 256-floor range.
 *
 * 为指定矿层生成程序化的 28×28 布局，同时排除该层固定入口和出口实体占用的
 * 格子。
 * 参数：mine_kind 选择 MINE_SPRING 或 MINE_LAKE；floor_index 是该矿场
 * 256 层范围内从 0 开始的层号。
 */
void GenerateMineFloorLayout(MaryMineKind mine_kind, int floor_index);

/*
 * Moves the player from the current spring- or lake-mine floor to the next
 * deeper floor and updates the active mine-floor state.
 * Parameters: none.
 * The calling script handles the fade and the one-minute time advance around
 * this operation.
 *
 * 将玩家从当前泉矿或湖矿楼层移动到下一层，并更新当前矿层状态。
 * 参数：无。调用脚本负责在该操作前后处理淡入淡出以及经过一分钟的时间更新。
 */
void DescendMineFloor(void);

/*
 * Returns whether the selected cursed tool's curse is still active. The
 * engine maps the six cursed tool IDs to six dedicated curse-state records.
 * Parameter: tool_id is one of the six cursed TOOL_* IDs. Return value:
 * nonzero while that tool remains cursed; zero otherwise.
 *
 * 返回所选诅咒农具的诅咒是否仍然生效。引擎会把六种诅咒农具 ID 映射到六个
 * 独立的诅咒状态记录。
 * 参数：tool_id 为六种诅咒 TOOL_* ID 之一。
 * 返回值：仍受诅咒时为非零，否则为零。
 */
MaryBool IsToolCursed(MaryToolId tool_id);

/*
 * Advances the selected cursed tool's lift condition. Returns 1 only when
 * this update completes the condition and lifts the curse; otherwise returns
 * 0. Event scripts use the result to present the curse-lift sequence.
 * Parameter: tool_id is one of the six cursed TOOL_* IDs.
 *
 * 推进所选诅咒农具的解除条件。仅当本次更新刚好完成条件并解除诅咒时返回 1，
 * 否则返回 0；事件脚本据此播放解除诅咒的演出。
 * 参数：tool_id 为六种诅咒 TOOL_* ID 之一。
 */
MaryBool AdvanceCursedToolLiftProgress(MaryToolId tool_id);

/*
 * Applies the church/confessional removal rule to the selected cursed tool.
 * Returns 1 when that attempt lifts the curse. This is the operation used
 * after the confessional script accepts the fee.
 * Parameter: tool_id is one of the six cursed TOOL_* IDs.
 *
 * 对所选诅咒农具应用教堂忏悔室的解除规则；本次尝试成功解除时返回 1。该函数
 * 用于忏悔室脚本确认并扣除费用之后。
 * 参数：tool_id 为六种诅咒 TOOL_* ID 之一。
 */
MaryBool AttemptChurchCursedToolRemoval(MaryToolId tool_id);

/*
 * Creates a temporary event icon in an event-local scene slot. Coordinates
 * use the current map's local pixel space, like entity placement and camera
 * movement; layer selects the scene display layer, and icon_id may
 * be supplied by GetFoodIconId() or as an original numeric engine icon ID.
 * Scripts may keep several slots alive simultaneously, so slot is an
 * event-local handle rather than an icon ID.
 *
 * 在带编号的场景槽中创建临时事件图标。x、y 与实体定位及镜头移动一样，使用
 * 当前地图的局部像素坐标；layer 选择场景显示层；icon_id 可由 GetFoodIconId()
 * 提供，也可使用原始引擎图标数字 ID。
 * 脚本可以同时保留多个槽，因此 slot 是事件局部句柄，而不是图标 ID。
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
 * Parameter: food_id is FOOD_* or the exact original food ID.
 * Return value: the engine icon ID associated with that food.
 *
 * 将食品 ID 映射为菜单与消息显示使用的图标。
 * 参数：food_id 为 FOOD_* 或精确原始食品 ID。
 * 返回值：该食品关联的引擎图标 ID。
 */
MaryEventIconId GetFoodIconId(MaryFoodId food_id);

/*
 * Maps an article ID to the icon used by menus and event presentation.
 * Parameter: article_id is ARTICLE_* or the exact original article ID.
 * Return value: the engine icon ID associated with that article.
 *
 * 将物品 ID 映射为菜单与事件显示使用的图标。
 * 参数：article_id 为 ARTICLE_* 或精确原始物品 ID。
 * 返回值：该物品关联的引擎图标 ID。
 */
MaryEventIconId GetArticleIconId(MaryArticleId article_id);

/*
 * Maps a tool ID to the icon used by menus and event presentation.
 * Parameter: tool_id is TOOL_* or the exact original tool ID.
 * Return value: the engine icon ID associated with that tool.
 *
 * 将工具 ID 映射为菜单与事件显示使用的图标。
 * 参数：tool_id 为 TOOL_* 或精确原始工具 ID。
 * 返回值：该工具关联的引擎图标 ID。
 */
MaryEventIconId GetToolIconId(MaryToolId tool_id);

/*
 * Verified retail no-op retained by the farming-tutorial bytecode. The VM
 * pushes five values that resemble grid x/y, tile state, object ID, and
 * variant, but all four ROMs dispatch this slot directly to the common return
 * block. The handler does not pop or inspect them and changes no field state.
 *
 * 种田教程字节码中保留的已确认零售版空操作。VM 会压入五个看似网格 x/y、
 * 格子状态、对象 ID 与变体的值，但四个 ROM 都把该槽直接派发到共用返回块；
 * handler 不弹出也不检查这些值，不会改变农田状态。
 */
void NoOpTutorialFieldTile(int x, int y, int tile_state, int object_id, int variant);

/*
 * Verified three-operand retail no-op adjacent to NoOpTutorialFieldTile. Its
 * coordinate-like operands are retained only to preserve the original stack
 * program and emitted bytes.
 *
 * 与 NoOpTutorialFieldTile 相邻的已确认三操作数零售版空操作。看似坐标与状态的
 * 操作数仅为保存原始栈程序及编译字节而保留。
 */
void NoOpTutorialFieldObject(int x, int y, int object_state);

/*
 * Verified three-operand retail no-op found in the chicken tutorial. The
 * coordinate- and slot-like values are not consumed by the native handler.
 *
 * 养鸡教程中的已确认三操作数零售版空操作。看似坐标与槽位的值不会被原生
 * handler 消费。
 */
void NoOpTutorialEggDefinition(int x, int y, int egg_slot);

/* Verified one-operand retail no-op paired with NoOpTutorialEggDefinition.
 *
 * 与 NoOpTutorialEggDefinition 配对的已确认单操作数零售版空操作。
 */
void NoOpTutorialEggSelection(int egg_slot);

/*
 * Puts the player into the scripted state for holding an actor graphic above
 * their head and waits for the transition.
 * Parameter: animation_id selects
 * the target-specific ANIMATION_ID_* graphic animation. Chicken tutorials call this after
 * removing the live chicken entity; FoMT and MFoMT use different IDs.
 *
 * 让玩家进入把角色图形举在头顶的脚本状态，并等待状态转场完成。
 * 参数：
 * animation_id 使用目标版本专用的 ANIMATION_ID_* 图形动画。养鸡教程会在删除活动鸡实体后调用
 * 本函数；FoMT 与 MFoMT 使用不同编号。
 */
void BeginHoldingActorGraphic(MaryAnimationId animation_id);

/*
 * Verified three-operand retail no-op found before animal animation setup in
 * tutorial scripts. The entity-, growth-, and actor-kind-like values are not
 * consumed; actual entity setup is performed by other engine paths.
 *
 * 教程脚本在设置动物动画前保留的已确认三操作数零售版空操作。看似实体、成长
 * 阶段和角色类别的值不会被消费；实际实体初始化由其他引擎路径完成。
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
 *
 * 返回事件派发器附加到当前运行脚本的整数上下文值，其含义由事件决定。已确认的
 * 用法包括：Zack 收货事件中的当天出货箱总值，以及女神累计出货事件中的产品 ID。
 * 不应将其理解为全局固定的“读取出货额”或“读取产品 ID”函数。
 */
int GetEventContextValue(void);

/*
 * Cycles backward through the tool rucksack until another cursed tool is
 * equipped, or all ten slots have been checked.
 * Parameters: none. The church
 * uses this after blessing the currently equipped cursed tool so a remaining
 * cursed tool, if any, becomes selected.
 *
 * 向后轮换工具背包，直到装备另一把诅咒农具，或检查完全部十个槽位。
 * 参数：无。教堂解除当前装备农具的诅咒后调用此函数，使仍存在的下一把诅咒
 * 农具自动成为当前选择。
 */
void SelectNextCursedTool(void);
#if defined(MARY_MFOMT)
/*
 * Tests whether an egg already occupies the chicken incubator selected by the
 * MFoMT breeding event. Parameters: none. Return value: TRUE while daily event
 * type 0x23 owns the incubator place; FALSE otherwise.
 *
 * 检查 MFoMT 繁殖事件选择的鸡蛋孵化位置是否已被占用。
 * 参数：无。返回值：每日事件类型 0x23 占用孵化位置时为 TRUE，否则为 FALSE。
 */
MaryBool IsChickenIncubatorOccupied(void);

/*
 * Tests whether a cow already occupies the barn pregnancy place selected by
 * the MFoMT breeding event. Parameters: none. Return value: TRUE while daily
 * event type 0x24 owns the place; FALSE otherwise.
 *
 * 检查 MFoMT 繁殖事件选择的牛妊娠位置是否已被占用。
 * 参数：无。返回值：每日事件类型 0x24 占用该位置时为 TRUE，否则为 FALSE。
 */
MaryBool IsCowPregnancySlotOccupied(void);

/*
 * Tests whether a sheep already occupies the barn pregnancy place selected by
 * the MFoMT breeding event. Parameters: none. Return value: TRUE while daily
 * event type 0x25 owns the place; FALSE otherwise.
 *
 * 检查 MFoMT 繁殖事件选择的羊妊娠位置是否已被占用。
 * 参数：无。返回值：每日事件类型 0x25 占用该位置时为 TRUE，否则为 FALSE。
 */
MaryBool IsSheepPregnancySlotOccupied(void);

/*
 * Returns the accumulated catch count from one MFoMT fishing-record slot.
 * Parameter: record_id is FISHING_RECORD_* in the complete 59-slot domain.
 * Return value: the first 32-bit field of the selected eight-byte record.
 * MFoMT-US and MFoMT-JP use the same layout; IDs 53-58 are River Kings.
 *
 * 返回 MFoMT 指定钓鱼记录槽中的累计捕获数量。
 * 参数：record_id 为完整 59 槽域中的 FISHING_RECORD_*。
 * 返回值：所选八字节记录的第一个 32 位字段。MFoMT-US 与 MFoMT-JP 使用相同
 * 布局；编号 53-58 为鱼王。
 */
int GetFishCatchCount(MaryFishingRecordId record_id);

/*
 * Returns the largest recorded size from one MFoMT fishing-record slot.
 * Parameter: record_id is FISHING_RECORD_* in the complete 59-slot domain.
 * Return value: the second 32-bit field of the selected eight-byte record, in
 * the engine's native fish-size unit.
 *
 * 返回 MFoMT 指定钓鱼记录槽中的最大捕获尺寸。
 * 参数：record_id 为完整 59 槽域中的 FISHING_RECORD_*。
 * 返回值：所选八字节记录的第二个 32 位字段，单位为引擎原生鱼尺寸单位。
 */
int GetLargestCaughtFishSize(MaryFishingRecordId record_id);

/*
 * Returns whether map_id exists in MFoMT's initialized map-metadata registry.
 * Both regional ROMs search a counted array of 20-byte records by the first
 * 16-bit field. The constructor folds 117 static map descriptors into 108
 * unique map records. This reports registry membership only; it does not mean
 * that story access to the map is currently unlocked.
 *
 * 返回 map_id 是否存在于 MFoMT 初始化后的地图元数据注册表。两套地区 ROM 均按
 * 首个 16 位字段搜索一组带计数的 20 字节记录；构造器会把 117 条静态地图描述
 * 合并成 108 个唯一地图记录。本函数只报告注册表成员关系，不表示剧情上已经
 * 解锁或当前能够进入该地图。
 */
MaryBool IsMapRegistered(MaryMapId map_id);

/*
 * Returns the accumulated 16-bit use experience for one of the six standard
 * tools.
 * Parameter: tool_kind is TOOL_KIND_* in the public sickle/hoe/axe/
 * hammer/watering-can/fishing-rod order. Invalid kinds return zero.
 *
 * 返回六种标准农具之一累计的 16 位使用经验。
 * 参数：tool_kind 使用 TOOL_KIND_*，对外顺序为镰刀、锄头、斧头、锤子、
 * 洒水壶、钓竿。无效类别返回零。
 */
int GetToolExperience(MaryToolKind tool_kind);

/*
 * Returns the number of animals in the selected family whose festival-winner
 * bit is set.
 * Parameter: animal_kind is ANIMAL_KIND_* in horse, cow, sheep,
 * chicken, and dog order. Invalid kinds return zero.
 *
 * 返回所选动物类别中已设置祭典优胜标志的动物数量。
 * 参数：animal_kind 使用 ANIMAL_KIND_*，顺序为马、牛、羊、鸡、狗；无效类别
 * 返回零。
 */
int CountFestivalWinningAnimals(MaryAnimalKind animal_kind);
#endif
