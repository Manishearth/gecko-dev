/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Main header first:
#include "nsCSSClipPathInstance.h"

#include "gfx2DGlue.h"
#include "gfxPlatform.h"
#include "mozilla/gfx/2D.h"
#include "mozilla/gfx/PathHelpers.h"
#include "nsCSSRendering.h"
#include "nsIFrame.h"
#include "nsRenderingContext.h"
#include "nsRuleNode.h"

using namespace mozilla;
using namespace mozilla::gfx;

/* static*/ void
nsCSSClipPathInstance::ApplyBasicShapeClip(gfxContext& aContext,
                                           nsIFrame* aFrame)
{
  auto& clipPathStyle = aFrame->StyleSVGReset()->mClipPath;
  StyleShapeSourceType type = clipPathStyle.GetType();
  MOZ_ASSERT(type != StyleShapeSourceType::None_, "unexpected none value");
  // In the future nsCSSClipPathInstance may handle <clipPath> references as
  // well. For the time being return early.
  if (type == StyleShapeSourceType::URL) {
    return;
  }

  nsCSSClipPathInstance instance(aFrame, clipPathStyle);

  aContext.NewPath();
  RefPtr<Path> path = instance.CreateClipPath(aContext.GetDrawTarget());
  aContext.SetPath(path);
  aContext.Clip();
}

/* static*/ bool
nsCSSClipPathInstance::HitTestBasicShapeClip(nsIFrame* aFrame,
                                             const gfxPoint& aPoint)
{
  auto& clipPathStyle = aFrame->StyleSVGReset()->mClipPath;
  StyleShapeSourceType type = clipPathStyle.GetType();
  MOZ_ASSERT(type != StyleShapeSourceType::None_, "unexpected none value");
  // In the future nsCSSClipPathInstance may handle <clipPath> references as
  // well. For the time being return early.
  if (type == StyleShapeSourceType::URL) {
    return false;
  }

  nsCSSClipPathInstance instance(aFrame, clipPathStyle);

  RefPtr<DrawTarget> drawTarget =
    gfxPlatform::GetPlatform()->ScreenReferenceDrawTarget();
  RefPtr<Path> path = instance.CreateClipPath(drawTarget);
  float pixelRatio = float(nsPresContext::AppUnitsPerCSSPixel()) /
                     aFrame->PresContext()->AppUnitsPerDevPixel();
  return path->ContainsPoint(ToPoint(aPoint) * pixelRatio, Matrix());
}

already_AddRefed<Path>
nsCSSClipPathInstance::CreateClipPath(DrawTarget* aDrawTarget)
{
  nsRect r;
  // XXXkrit SVG needs to use different boxes.
  switch (mClipPathStyle.GetReferenceBox()) {
    case StyleClipPathGeometryBox::Content:
      r = mTargetFrame->GetContentRectRelativeToSelf();
      break;
    case StyleClipPathGeometryBox::Padding:
      r = mTargetFrame->GetPaddingRectRelativeToSelf();
      break;
    case StyleClipPathGeometryBox::Margin:
      r = mTargetFrame->GetMarginRectRelativeToSelf();
      break;
    default: // Use the border box
      r = mTargetFrame->GetRectRelativeToSelf();
  }

  if (mClipPathStyle.GetType() != StyleShapeSourceType::Shape) {
    // TODO Clip to border-radius/reference box if no shape
    // was specified.
    RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder();
    return builder->Finish();
  }

  nscoord appUnitsPerDevPixel =
    mTargetFrame->PresContext()->AppUnitsPerDevPixel();
  r = ToAppUnits(r.ToNearestPixels(appUnitsPerDevPixel), appUnitsPerDevPixel);

  StyleBasicShape* basicShape = mClipPathStyle.GetBasicShape();
  switch (basicShape->GetShapeType()) {
    case StyleBasicShapeType::Circle:
      return CreateClipPathCircle(aDrawTarget, r);
    case StyleBasicShapeType::Ellipse:
      return CreateClipPathEllipse(aDrawTarget, r);
    case StyleBasicShapeType::Polygon:
      return CreateClipPathPolygon(aDrawTarget, r);
    case StyleBasicShapeType::Inset:
      return CreateClipPathInset(aDrawTarget, r);
      break;
    default:
      MOZ_MAKE_COMPILER_ASSUME_IS_UNREACHABLE("Unexpected shape type");
  }
  // Return an empty Path:
  RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder();
  return builder->Finish();
}

static void
EnumerationToLength(nscoord& aCoord, int32_t aType,
                    nscoord aCenter, nscoord aPosMin, nscoord aPosMax)
{
  nscoord dist1 = abs(aPosMin - aCenter);
  nscoord dist2 = abs(aPosMax - aCenter);
  switch (aType) {
    case NS_RADIUS_FARTHEST_SIDE:
      aCoord = dist1 > dist2 ? dist1 : dist2;
      break;
    case NS_RADIUS_CLOSEST_SIDE:
      aCoord = dist1 > dist2 ? dist2 : dist1;
      break;
    default:
      NS_NOTREACHED("unknown keyword");
      break;
  }
}

already_AddRefed<Path>
nsCSSClipPathInstance::CreateClipPathCircle(DrawTarget* aDrawTarget,
                                            const nsRect& aRefBox)
{
  StyleBasicShape* basicShape = mClipPathStyle.GetBasicShape();

  RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder();

  nsPoint topLeft, anchor;
  nsSize size = nsSize(aRefBox.width, aRefBox.height);
  nsImageRenderer::ComputeObjectAnchorPoint(basicShape->GetPosition(),
                                            size, size,
                                            &topLeft, &anchor);
  Point center = Point(anchor.x + aRefBox.x, anchor.y + aRefBox.y);

  const nsTArray<nsStyleCoord>& coords = basicShape->Coordinates();
  MOZ_ASSERT(coords.Length() == 1, "wrong number of arguments");
  float referenceLength = sqrt((aRefBox.width * aRefBox.width +
                                aRefBox.height * aRefBox.height) / 2.0);
  nscoord r = 0;
  if (coords[0].GetUnit() == eStyleUnit_Enumerated) {
    nscoord horizontal, vertical;
    EnumerationToLength(horizontal, coords[0].GetIntValue(),
                        center.x, aRefBox.x, aRefBox.x + aRefBox.width);
    EnumerationToLength(vertical, coords[0].GetIntValue(),
                        center.y, aRefBox.y, aRefBox.y + aRefBox.height);
    if (coords[0].GetIntValue() == NS_RADIUS_FARTHEST_SIDE) {
      r = horizontal > vertical ? horizontal : vertical;
    } else {
      r = horizontal < vertical ? horizontal : vertical;
    }
  } else {
    r = nsRuleNode::ComputeCoordPercentCalc(coords[0], referenceLength);
  }

  nscoord appUnitsPerDevPixel =
    mTargetFrame->PresContext()->AppUnitsPerDevPixel();
  builder->Arc(center / appUnitsPerDevPixel, r / appUnitsPerDevPixel,
               0, Float(2 * M_PI));
  builder->Close();
  return builder->Finish();
}

already_AddRefed<Path>
nsCSSClipPathInstance::CreateClipPathEllipse(DrawTarget* aDrawTarget,
                                             const nsRect& aRefBox)
{
  StyleBasicShape* basicShape = mClipPathStyle.GetBasicShape();

  RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder();

  nsPoint topLeft, anchor;
  nsSize size = nsSize(aRefBox.width, aRefBox.height);
  nsImageRenderer::ComputeObjectAnchorPoint(basicShape->GetPosition(),
                                            size, size,
                                            &topLeft, &anchor);
  Point center = Point(anchor.x + aRefBox.x, anchor.y + aRefBox.y);

  const nsTArray<nsStyleCoord>& coords = basicShape->Coordinates();
  MOZ_ASSERT(coords.Length() == 2, "wrong number of arguments");
  nscoord rx = 0, ry = 0;
  if (coords[0].GetUnit() == eStyleUnit_Enumerated) {
    EnumerationToLength(rx, coords[0].GetIntValue(),
                        center.x, aRefBox.x, aRefBox.x + aRefBox.width);
  } else {
    rx = nsRuleNode::ComputeCoordPercentCalc(coords[0], aRefBox.width);
  }
  if (coords[1].GetUnit() == eStyleUnit_Enumerated) {
    EnumerationToLength(ry, coords[1].GetIntValue(),
                        center.y, aRefBox.y, aRefBox.y + aRefBox.height);
  } else {
    ry = nsRuleNode::ComputeCoordPercentCalc(coords[1], aRefBox.height);
  }

  nscoord appUnitsPerDevPixel =
    mTargetFrame->PresContext()->AppUnitsPerDevPixel();
  EllipseToBezier(builder.get(),
                  center / appUnitsPerDevPixel,
                  Size(rx, ry) / appUnitsPerDevPixel);
  builder->Close();
  return builder->Finish();
}

already_AddRefed<Path>
nsCSSClipPathInstance::CreateClipPathPolygon(DrawTarget* aDrawTarget,
                                             const nsRect& aRefBox)
{
  StyleBasicShape* basicShape = mClipPathStyle.GetBasicShape();
  const nsTArray<nsStyleCoord>& coords = basicShape->Coordinates();
  MOZ_ASSERT(coords.Length() % 2 == 0 &&
             coords.Length() >= 2, "wrong number of arguments");

  FillRule fillRule = basicShape->GetFillRule() == StyleFillRule::Nonzero ?
                        FillRule::FILL_WINDING : FillRule::FILL_EVEN_ODD;
  RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder(fillRule);

  nscoord x = nsRuleNode::ComputeCoordPercentCalc(coords[0], aRefBox.width);
  nscoord y = nsRuleNode::ComputeCoordPercentCalc(coords[1], aRefBox.height);
  nscoord appUnitsPerDevPixel =
    mTargetFrame->PresContext()->AppUnitsPerDevPixel();
  builder->MoveTo(Point(aRefBox.x + x, aRefBox.y + y) / appUnitsPerDevPixel);
  for (size_t i = 2; i < coords.Length(); i += 2) {
    x = nsRuleNode::ComputeCoordPercentCalc(coords[i], aRefBox.width);
    y = nsRuleNode::ComputeCoordPercentCalc(coords[i + 1], aRefBox.height);
    builder->LineTo(Point(aRefBox.x + x, aRefBox.y + y) / appUnitsPerDevPixel);
  }
  builder->Close();
  return builder->Finish();
}

already_AddRefed<Path>
nsCSSClipPathInstance::CreateClipPathInset(DrawTarget* aDrawTarget,
                                           const nsRect& aRefBox)
{
  StyleBasicShape* basicShape = mClipPathStyle.GetBasicShape();
  const nsTArray<nsStyleCoord>& coords = basicShape->Coordinates();
  MOZ_ASSERT(coords.Length() == 4, "wrong number of arguments");

  RefPtr<PathBuilder> builder = aDrawTarget->CreatePathBuilder();

  nscoord appUnitsPerDevPixel =
    mTargetFrame->PresContext()->AppUnitsPerDevPixel();
  nscoord top = nsRuleNode::ComputeCoordPercentCalc(coords[0], aRefBox.height);
  nscoord right = nsRuleNode::ComputeCoordPercentCalc(coords[1], aRefBox.width);
  nscoord bottom = nsRuleNode::ComputeCoordPercentCalc(coords[2],
                                                       aRefBox.height);
  nscoord left = nsRuleNode::ComputeCoordPercentCalc(coords[3], aRefBox.width);

  Point anchor =
      Point(aRefBox.x + left, aRefBox.y + top) / appUnitsPerDevPixel;
  Size size = Size(aRefBox.width - left - right,
                   aRefBox.height - top - bottom) / appUnitsPerDevPixel;
  Rect rect(anchor, size);

  nsStyleCorners& radius = basicShape->GetRadius();

  if (nsLayoutUtils::HasNonZeroCorner(radius)) {
    Size sizes[4]; // tl, tr, br, bl
    NS_FOR_CSS_FULL_CORNERS(corner) {
      int cx = NS_FULL_TO_HALF_CORNER(corner, false);
      int cy = NS_FULL_TO_HALF_CORNER(corner, true);
      nscoord x = nsRuleNode::ComputeCoordPercentCalc(radius.Get(cx),
                                                      aRefBox.width);
      nscoord y = nsRuleNode::ComputeCoordPercentCalc(radius.Get(cy),
                                                      aRefBox.height);
      sizes[corner] = Size(x, y) / appUnitsPerDevPixel;
    }

    RectCornerRadii corners(sizes[0], sizes[1], sizes[2], sizes[3]);

    AppendRoundedRectToPath(builder, rect, corners, true);
  } else {
    AppendRectToPath(builder, rect, true);
  }
  return builder->Finish();
}
